use crate::{
    common::AppError,
    dto::order::{DeliverToMarketDTO, ExchangeDTO, ExchangeItemQuantityUpdateRequest, OrderDeliveryHistoryResponse},
    map_db_err,
    models::{
        claims::Claims, order_action::OrderAction, order_machine::OrderStateMachine,
        order_status::OrderStatus, tenant_type::TenantType,
    },
    repositories::my_sql_repository::MySqlRepository,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::{debug, error};

#[async_trait]
pub(crate) trait ProviderOrderRepository: Send + Sync {
    /// Mark the order as processing by the provider
    ///
    /// If the order is in the Assigned state, the delivery staff will be assigned to the order.
    /// If the order is in the ExchangeRequested state, the exchange will be started.
    ///
    /// # Arguments
    /// * `order_code` - The code of the order
    /// * `operator` - The operator of the order
    /// * `provider_hash` - The hash of the provider
    /// * `delivery_staff_id` - The id of the delivery staff
    ///
    /// # Returns
    /// * `Result<OrderStatus, AppError>` - The result of the operation.
    ///
    /// # Errors
    /// * `AppError` - The error of the operation.
    async fn start_preparing_order(
        &self,
        order_code: &str,
        claims: &Claims,
        delivery_staff_id: Option<&str>,
    ) -> Result<OrderStatus, AppError>;

    async fn deliver_to_market(
        &self,
        order_code: &str,
        claims: &Claims,
        deliver_to_market_dto: &DeliverToMarketDTO,
    ) -> Result<(), AppError>;

    async fn exchange_deliver_to_market(
        &self,
        order_code: &str,
        operator: &str,
        provider_hash: &str,
        exchange_dto: &ExchangeDTO,
    ) -> Result<(), AppError>;

    /// This method is used to update the actual quantity of the exchange item when the return goods is processed.
    /// It does not update the order details table and orders table.
    ///
    /// # Arguments
    /// * `order_detail_id` - The id of the order detail.
    /// * `operator` - The operator of the return goods.
    /// * `exchange_item_update_dto` - The exchange item update dto.
    ///
    /// # Returns
    /// * `Result<(), AppError>` - The result of the operation.
    ///
    /// # Errors
    /// * `AppError` - The error of the operation.
    ///
    /// # Examples
    /// ```rust
    /// let exchange_item_update_dto = ExchangeItemUpdateDTO {
    ///     order_detail_id: 1,
    ///     actual_quantity: 10,
    /// };
    /// let result = repository.update_exchange_item_quantity("operator", &exchange_item_update_dto);
    /// ```
    async fn update_exchange_item_quantity(
        &self,
        order_code: &str,
        order_detail_id: i32,
        claims: &Claims,
        exchange_item_update_dto: &ExchangeItemQuantityUpdateRequest,
    ) -> Result<(), AppError>;


    // 获取订单配送历史
    async fn get_order_delivery_history(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<OrderDeliveryHistoryResponse, AppError>;
}

#[async_trait]
impl ProviderOrderRepository for MySqlRepository {
    async fn start_preparing_order(
        &self,
        order_code: &str,
        claims: &Claims,
        delivery_staff_id: Option<&str>,
    ) -> Result<OrderStatus, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        let (order_id, order_status_str, assignment_id, _, _) = self
            .fetch_provider_order(
                claims.tenant_hash.as_str(),
                order_code,
                &format!("订单 {} 没有找到，不能备货", order_code),
            )
            .await?;

        let current_status = OrderStatus::try_from(order_status_str.as_str())?;
        let invalid_state_msg = format!("订单 {} 状态错误，不能进行备货", order_code);

        let action = match current_status {
            OrderStatus::Assigned => OrderAction::StartPreparing,
            OrderStatus::ExchangeRequested => OrderAction::StartPreparing,
            _ => return Err(AppError::Validation(invalid_state_msg)),
        };

        let next = OrderStateMachine::next_state(current_status, action, TenantType::Provider)
            .map_err(|err| {
                error!("{err}");
                AppError::Validation(format!("订单 {} 状态错误，不能进行备货", order_code))
            })?;

        if current_status == OrderStatus::Assigned {
            let id_card = delivery_staff_id
                .ok_or_else(|| AppError::Validation("配送员 ID 不能为空".to_string()))?;

            // Insert provider delivery basic information
            let insert_result = sqlx::query!(
                r#"INSERT INTO provider_deliveries (assignment_id, delivered_by, delivery_contact_number) 
                   SELECT ? AS assignment_id, name, phone 
                   FROM delivery_staff 
                   WHERE id_card = ? AND status = true"#,
                assignment_id,
                id_card
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to insert provider delivery"))?;

            if insert_result.rows_affected() == 0 {
                return Err(AppError::NotFound(format!("配送员 {} 没有找到", id_card)));
            }

            let provider_delivery_id = insert_result.last_insert_id();

            // insert into order delivery items table
            sqlx::query!(
                r#"INSERT INTO provider_delivery_items (delivery_id, order_detail_id, product_code, unit_price, weight_unit) 
                SELECT ? AS delivery_id, od.id AS order_detail_id, od.product_code, od.discounted_unit_price, od.unit AS weight_unit
                FROM order_details od WHERE od.order_id = ?"#,
                provider_delivery_id,
                order_id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to insert provider delivery item"))?;
        } else {
            // ExchangeRequested state, insert new provider delivery message
            let provider_delivery = sqlx::query!(
                r#"
                SELECT id, delivered_by, delivery_contact_number, delivery_round
                FROM provider_deliveries
                WHERE assignment_id = ?
                ORDER BY delivery_round DESC
                LIMIT 1
                "#,
                order_id
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to get provider delivery"))?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Provider delivery not found for assignment id {}",
                    order_id
                ))
            })?;

            // calculate the next delivery round
            let next_round = provider_delivery.delivery_round + 1;

            // insert the next delivery record
            sqlx::query!(
                r#"
                INSERT INTO provider_deliveries
                (assignment_id, delivered_by, delivery_contact_number, parent_id, delivery_round, delivery_type)
                VALUES (?, ?, ?, ?, ?, 'EXCHANGE')
                "#,
                order_id,
                provider_delivery.delivered_by,
                provider_delivery.delivery_contact_number,
                provider_delivery.id,
                next_round
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to insert next round provider delivery"))?;

            let provider_delivery_id = sqlx::query!("SELECT LAST_INSERT_ID() as id")
                .fetch_one(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to get last insert id"))?
                .id;
            // insert into order delivery items table
            // 只插入需要换货的商品（最近一轮验收中标记为EXCHANGE的商品）
            sqlx::query!(
                r#"INSERT INTO provider_delivery_items (delivery_id, order_detail_id, product_code, unit_price, weight_unit)
                SELECT ? AS delivery_id, od.id AS order_detail_id, od.product_code, od.discounted_unit_price, od.unit AS weight_unit
                FROM order_details od
                INNER JOIN order_inspection_items oii ON od.id = oii.order_detail_id
                WHERE od.order_id = ?
                  AND oii.inspection_id IN (
                    SELECT oi.id FROM order_inspections oi
                    WHERE oi.order_id = ?
                      AND oi.inspection_round = (
                        SELECT MAX(inspection_round) FROM order_inspections
                        WHERE order_id = ? AND inspection_result = 'EXCHANGE'
                      )
                      AND oi.inspection_result = 'EXCHANGE'
                  )
                  AND oii.result = 'EXCHANGE'"#,
                provider_delivery_id,
                order_id,
                order_id,
                order_id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to insert provider delivery item"))?;
            // If has any return goods, update the return goods status to RETURNED means the return goods request is accepted
            sqlx::query!(
                r#"UPDATE return_exchange_records 
                   SET status = 'RETURNED', processed_by = ?, processed_at = NOW() 
                   WHERE operation_type = 'RETURN'
                     AND order_detail_id IN (
                       SELECT id FROM order_details WHERE order_id = ?
                   )"#,
                claims.real_name.as_str(),
                order_id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to update returned items status"))?;
        }

        // Update order status to SupplierPreparing or ExchangeInProgress
        let update_result = sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ? AND order_status = ?"#,
            next.to_str(),
            order_id,
            &order_status_str
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!(
            "Failed to update order status from ExchangeRequested to ExchangeInProgress"
        ))?;

        if update_result.rows_affected() == 0 {
            let _ = tx.rollback().await;
            return Err(AppError::Conflict(format!(
                "订单 {} 状态已变更，请刷新后重试",
                order_code
            )));
        }

        // insert order status history
        self.insert_order_status_history(
            &mut tx,
            order_id,
            order_status_str.as_str(),
            next,
            claims.real_name.as_str(),
            action,
        )
        .await?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        Ok(next)
    }

    /// Deliver the order to the market when the order is in the SupplierPreparing state
    async fn deliver_to_market(
        &self,
        order_code: &str,
        claims: &Claims,
        deliver_to_market_dto: &DeliverToMarketDTO,
    ) -> Result<(), AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction to update order status from SupplierPreparing to SupplierDelivering"))?;

        // Use the shared helper method to fetch provider's order
        let (order_id, order_status_str, _, delivery_id, _) = self
            .fetch_provider_order(
                claims.tenant_hash.as_str(),
                order_code,
                &format!("订单 {} 没有找到", order_code),
            )
            .await?;

        let current_status = OrderStatus::try_from(order_status_str.as_str())?;
        let invalid_state_msg = format!("订单 {} 状态错误，不能配送到市场", order_code);

        let action = match current_status {
            OrderStatus::SupplierPreparing => OrderAction::DeliverToMarket,
            OrderStatus::ExchangeInProgress => OrderAction::DeliverToMarket,
            _ => return Err(AppError::validation(invalid_state_msg)),
        };

        let next = OrderStateMachine::next_state(current_status, action, TenantType::Provider)
            .map_err(|err| {
                error!("{err}");
                AppError::validation(format!("订单 {} 状态错误，不能配送到市场", order_code))
            })?;

        // update actual quantity and actual amount in order_details table
        for item in deliver_to_market_dto.items.iter() {
            let id = item.id;
            let delivered_quantity = item.delivered_quantity;

            sqlx::query!(r#"
             UPDATE provider_delivery_items SET actual_qty = ? WHERE delivery_id = ? AND order_detail_id = ?
            "#,
                delivered_quantity,
                delivery_id,
                id
            )
            .execute(&mut *tx)
            .await.map_err(map_db_err!("Failed to update provider delivery item"))?;
        };

        // update provider delivery status to DELIVERED and delivered_at to now
        sqlx::query!(
            r#"UPDATE provider_deliveries SET delivery_status = 'DELIVERED', delivered_at = NOW() WHERE id = ?"#,
            delivery_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update provider delivery status"))?;

        // update order status
        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ? AND order_status = ?"#,
            next.to_str(),
            order_id,
            &order_status_str
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        // insert order status history
        self.insert_order_status_history(
            &mut tx,
            order_id,
            order_status_str.as_str(),
            next,
            claims.real_name.as_str(),
            action,
        )
        .await?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(())
    }

    async fn exchange_deliver_to_market(
        &self,
        order_code: &str,
        operator: &str,
        provider_hash: &str,
        exchange_dto: &ExchangeDTO,
    ) -> Result<(), AppError> {
        debug!(
            "Provider {} exchanges deliver to market for order code {} and exchange dto {:?}",
            operator, order_code, exchange_dto
        );
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        let (order_id, order_status_str, _, delivery_id, _) = self
            .fetch_provider_order(
                provider_hash,
                order_code,
                &format!("订单 {} 没有找到", order_code),
            )
            .await?;

        // Validate and transition order state
        let next = Self::validate_and_transition_order_state(
            order_status_str.as_str(),
            OrderAction::DeliverToMarket,
            TenantType::Provider,
            &format!(
                "Order status is not correct: {} -> {}",
                order_status_str,
                OrderStatus::ExchangeDelivering.to_str()
            )
            .as_str(),
        )?;

        let delivery_id = delivery_id.ok_or_else(|| {
            AppError::NotFound(format!(
                "订单 {} 没有找到状态为 PREPARING 的配送记录",
                order_code
            ))
        })?;
        debug!(
            "Exchange deliver to market for order code {} and delivery id {} and next status {}",
            order_code, delivery_id, next.to_str()
        );

        for item in exchange_dto.items.iter() {
            let id = item.id;
            let actual_quantity = item.actual_quantity;

            debug!(
                "Exchange deliver to market for order code {} and delivery id {} and item id {} and actual quantity {}",
                order_code, delivery_id, id, actual_quantity
            );

            let insert_result = sqlx::query!(r#"
              UPDATE provider_delivery_items SET actual_qty = ? WHERE delivery_id = ? AND order_detail_id = ?
            "#,
                actual_quantity,
                delivery_id,
                id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to insert provider delivery item"))?;

            if insert_result.rows_affected() == 0 {
                return Err(AppError::NotFound(format!(
                    "订单 {} 中没有找到订单明细 {}，无法创建换货配送明细",
                    order_code, id
                )));
            }
        }

        // update provider delivery status to DELIVERED and delivered_at to now
        sqlx::query!(
            r#"UPDATE provider_deliveries SET delivery_status = 'DELIVERED', delivered_at = NOW() WHERE id = ?"#,
            delivery_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update provider delivery status"))?;

        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ?"#,
            next.to_str(),
            order_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        self.insert_order_status_history(
            &mut tx,
            order_id,
            order_status_str.as_str(),
            next,
            operator,
            OrderAction::DeliverToMarket,
        )
        .await?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(())
    }

    async fn update_exchange_item_quantity(
        &self,
        order_code: &str,
        order_detail_id: i32,
        claims: &Claims,
        exchange_item_update_dto: &ExchangeItemQuantityUpdateRequest,
    ) -> Result<(), AppError> {
        debug!(
            "Provider {} updates exchange item actual quantity for order detail id {} and actual quantity {}",
            claims.real_name, order_detail_id, exchange_item_update_dto.actual_quantity
        );

        let (order_id, _, _, delivery_id, _) = self
            .fetch_provider_order(
                claims.tenant_hash.as_str(),
                order_code,
                &format!("订单 {} 没有找到", order_code),
            )
            .await?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        let exchange_item_id = sqlx::query!(
            r#"SELECT id FROM return_exchange_records WHERE order_detail_id = ? FOR UPDATE"#,
            order_detail_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get return exchange record"))?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Return exchange record not found: {}",
                order_detail_id
            ))
        })?;

        sqlx::query!(
            r#"UPDATE return_exchange_records SET actual_quantity = ?, status = 'PROGRESSED', processed_by = ?, processed_at = NOW()
             WHERE order_detail_id = ?"#,
            exchange_item_update_dto.actual_quantity, claims.real_name.as_str(), order_detail_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update return exchange record"))?;

        sqlx::query!(
            r#"UPDATE exchange_items SET shipped_at = NOW(), status = 'PROGRESSED'
             WHERE return_exchange_id = ?"#,
            exchange_item_id.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update exchange item status"))?;


    // insert into order delivery items table
    sqlx::query!(
        r#"INSERT INTO provider_delivery_items (delivery_id, order_detail_id, product_code, actual_qty, unit_price, weight_unit) 
        SELECT ? AS delivery_id, od.id AS order_detail_id, od.product_code, ? AS actual_qty, od.discounted_unit_price, od.unit AS weight_unit
        FROM order_details od WHERE od.id = ? AND od.order_id = ?"#,
        delivery_id,
        exchange_item_update_dto.actual_quantity,
        order_detail_id,
        order_id
    )
    .execute(&mut *tx)
    .await
    .map_err(map_db_err!("Failed to insert provider delivery item"))?;

    // update provider delivery status to DELIVERED and delivered_at to now
    sqlx::query!(
        r#"UPDATE provider_deliveries SET delivery_status = 'DELIVERED', delivered_at = NOW() WHERE id = ?"#,
        delivery_id
    )
    .execute(&mut *tx)
    .await
    .map_err(map_db_err!("Failed to update provider delivery status"))?;

    tx.commit()
        .await
        .map_err(map_db_err!("Failed to commit transaction"))?;
    Ok(())
    }

    async fn get_order_delivery_history(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<OrderDeliveryHistoryResponse, AppError> {
        // 获取订单信息
        let order_info = sqlx::query!(
            r#"
            SELECT o.id, o.order_code, o.order_status
            FROM orders o
            WHERE o.order_code = ?
            "#,
            order_code
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order"))?;

        // 查询所有已交付的记录 (delivery_status = 'DELIVERED')
        let deliveries = sqlx::query!(
            r#"
            SELECT pd.id, pd.delivery_round, pd.delivery_type, pd.delivery_status,
                   pd.delivered_at, pd.delivered_by, ds.name as staff_name, ds.phone as staff_phone,
                   oi.inspection_result, oi.inspected_at
            FROM provider_deliveries pd
            LEFT JOIN delivery_staff ds ON pd.delivered_by = ds.name
            LEFT JOIN order_inspections oi ON oi.order_id = ? AND oi.inspection_round = pd.delivery_round
            WHERE pd.assignment_id = ? AND pd.delivery_status = 'DELIVERED'
            ORDER BY pd.delivery_round ASC
            "#,
            order_info.id,
            order_info.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get deliveries"))?;

        let mut history_items = Vec::new();

        for delivery in deliveries {
            // 获取该交付轮次的商品信息
            let delivery_items = sqlx::query!(
                r#"
                SELECT pdi.order_detail_id, od.product_code, od.product_name,
                       od.category_id, od.category_name, od.unit, od.unit_price,
                       od.ordered_qty, pdi.actual_qty,
                       oii.inspected_qty,
                       oii.result as last_inspection_result, oii.created_at as inspection_at,
                       pdi.remark
                FROM provider_delivery_items pdi
                INNER JOIN order_details od ON pdi.order_detail_id = od.id
                LEFT JOIN order_inspection_items oii ON oii.order_detail_id = pdi.order_detail_id
                    AND oii.inspection_id = (
                        SELECT MAX(id) FROM order_inspections
                        WHERE order_id = ? AND inspection_round = ?
                    )
                WHERE pdi.delivery_id = ?
                ORDER BY pdi.order_detail_id ASC
                "#,
                order_info.id,
                delivery.delivery_round,
                delivery.id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get delivery items"))?;

            let items: Vec<_> = delivery_items
                .into_iter()
                .map(|item| crate::dto::order::OrderDeliveryHistoryItemDetail {
                    order_detail_id: item.order_detail_id as i32,
                    product_code: item.product_code,
                    product_name: item.product_name,
                    category_id: item.category_id,
                    category_name: item.category_name,
                    unit: item.unit,
                    unit_price: item.unit_price,
                    ordered_qty: item.ordered_qty,
                    need_to_deliver_qty: item.ordered_qty, // 使用ordered_qty作为默认值
                    actual_qty: Some(item.actual_qty),
                    inspected_qty: item.inspected_qty,
                    last_inspection_result: item.last_inspection_result,
                    inspection_at: item.inspection_at,
                    remark: item.remark,
                })
                .collect();

            let delivery_staff = if delivery.staff_name.is_some() && delivery.staff_phone.is_some() {
                Some(crate::dto::order::DeliveryStaffInfo {
                    id: delivery.delivered_by.clone(),
                    name: delivery.staff_name.unwrap(),
                    phone: delivery.staff_phone.unwrap(),
                })
            } else {
                None
            };

            history_items.push(crate::dto::order::OrderDeliveryHistoryItem {
                round: delivery.delivery_round,
                delivery_type: delivery.delivery_type.to_string(),
                delivery_status: delivery.delivery_status.to_string(),
                inspection_status: delivery.inspection_result.unwrap_or_else(|| "PENDING".to_string()),
                delivered_at: delivery.delivered_at.map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)).unwrap_or_default(),
                inspected_at: delivery.inspected_at.map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)),
                delivery_staff,
                items,
            });
        }

        Ok(OrderDeliveryHistoryResponse {
            order_code: order_info.order_code,
            history: history_items,
        })
    }
}
