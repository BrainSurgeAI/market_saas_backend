use crate::{
    common::AppError,
    dto::order::{
        DeliverToMarketDTO, ExchangeDTO, ExchangeItemQuantityUpdateRequest,
        OrderDeliveryHistoryResponse, ProviderDashboardStatsDTO, ProviderTodayDeliveredProductsDTO,
    },
    map_db_err,
    models::{
        claims::Claims, order_action::OrderAction, order_machine::OrderStateMachine,
        order_status::OrderStatus, tenant_type::TenantType,
    },
    repositories::my_sql_repository::MySqlRepository,
};

use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::FromRow;
use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error};

// 用于查询配送统计的临时结构
#[derive(Debug, FromRow)]
struct DeliveryStatsRow {
    delivering_count: i64,
    completed_count: i64,
}

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

    async fn get_provider_dashboard_stats(
        &self,
        provider_hash: &str,
        delivery_date: Option<NaiveDate>,
    ) -> Result<ProviderDashboardStatsDTO, AppError>;

    /// 获取 PROVIDER 今天已交付的商品聚合数据
    ///
    /// 返回今天（orders.delivery_date = 今天）所有订单对应的 provider_deliveries
    /// 的 delivery_status 是 'DELIVERED' 的商品聚合数据
    /// 根据 order_details 表的 id 与 provider_delivery_items 的 order_detail_id 关联，
    /// 累加 actual_qty
    async fn get_provider_today_delivered_products(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<ProviderTodayDeliveredProductsDTO>, AppError>;
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
        }

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

        //check exchange_dto.items is not empty
        if exchange_dto.items.is_empty() {
            return Err(AppError::NotFound(format!(
                "订单 {} 没有找到换货商品",
                order_code
            )));
        }

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
            order_code,
            delivery_id,
            next.to_str()
        );

        // 批量更新 provider_delivery_items，使用 CASE WHEN 提高效率
        if !exchange_dto.items.is_empty() {
            let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
                "UPDATE provider_delivery_items SET actual_qty = CASE order_detail_id ",
            );

            // 构建 CASE WHEN 语句
            for item in exchange_dto.items.iter() {
                debug!(
                    "Exchange deliver to market for order code {} and delivery id {} and item id {} and actual quantity {}",
                    order_code, delivery_id, item.id, item.actual_quantity
                );
                builder
                    .push("WHEN ")
                    .push_bind(item.id)
                    .push(" THEN ")
                    .push_bind(item.actual_quantity)
                    .push(" ");
            }

            builder.push("END WHERE delivery_id = ");
            builder.push_bind(delivery_id);
            builder.push(" AND order_detail_id IN (");

            let mut separated = builder.separated(", ");
            for item in exchange_dto.items.iter() {
                separated.push_bind(item.id);
            }
            separated.push_unseparated(")");

            let update_result = builder
                .build()
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!(
                    "Failed to batch update provider delivery items"
                ))?;

            // 验证所有记录都被更新了
            if update_result.rows_affected() != exchange_dto.items.len() as u64 {
                return Err(AppError::NotFound(format!(
                    "订单 {} 中部分订单明细未找到，无法更新换货配送明细。期望更新 {} 条，实际更新 {} 条",
                    order_code, exchange_dto.items.len(), update_result.rows_affected()
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

        // update return_exchange_records status to PROGRESSED and processed_at to now
        // 只更新 exchange_dto.items 中指定的 order_detail_id
        // 使用 CASE WHEN 为每个 order_detail_id 设置不同的 actual_quantity
        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
            "UPDATE return_exchange_records SET status = 'PROGRESSED', processed_at = NOW(), processed_by = "
        );
        
        builder.push_bind(operator);
        builder.push(", delivered_qty = CASE order_detail_id ");
        
        // 构建 CASE WHEN 语句，为每个 order_detail_id 设置对应的 actual_quantity
        for item in exchange_dto.items.iter() {
            builder.push("WHEN ")
                .push_bind(item.id)
                .push(" THEN ")
                .push_bind(item.actual_quantity)
                .push(" ");
        }
        
        builder.push("END WHERE order_detail_id IN (");
        
        let mut separated = builder.separated(", ");
        for item in exchange_dto.items.iter() {
            separated.push_bind(item.id);
        }
        separated.push_unseparated(")");

        builder
            .build()
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!(
                "Failed to update return exchange records status"
            ))?;

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
            "******Provider {} updates exchange item actual quantity for order detail id {} and actual quantity {}",
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
            r#"UPDATE return_exchange_records SET delivered_qty = ?, status = 'PROGRESSED', processed_by = ?, processed_at = NOW()
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
        _claims: &Claims,
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

            let delivery_staff = if delivery.staff_name.is_some() && delivery.staff_phone.is_some()
            {
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
                inspection_status: delivery
                    .inspection_result
                    .unwrap_or_else(|| "PENDING".to_string()),
                delivered_at: delivery
                    .delivered_at
                    .map(|dt| {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                    })
                    .unwrap_or_default(),
                inspected_at: delivery.inspected_at.map(|dt| {
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                }),
                delivery_staff,
                items,
            });
        }

        Ok(OrderDeliveryHistoryResponse {
            order_code: order_info.order_code,
            history: history_items,
        })
    }

    async fn get_provider_dashboard_stats(
        &self,
        provider_hash: &str,
        delivery_date: Option<NaiveDate>,
    ) -> Result<ProviderDashboardStatsDTO, AppError> {
        use sqlx::{MySql, QueryBuilder};
        use tracing::debug;

        debug!(
            "Fetching dashboard stats for provider_hash: {}, delivery_date: {:?}",
            provider_hash, delivery_date
        );

        // 查询待配送订单总数（ASSIGNED 或 SUPPLIER_PREPARING）
        let mut pending_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT COUNT(DISTINCT o.id) as count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            WHERE t.name_hash = 
            "#,
        );

        pending_query.push_bind(provider_hash);
        pending_query.push(" AND t.tenant_type = 'PROVIDER'");
        pending_query.push(" AND t.deleted_at IS NULL");
        pending_query.push(" AND o.order_status IN ('ASSIGNED', 'SUPPLIER_PREPARING')");
        pending_query.push(" AND o.deleted_at IS NULL");

        if let Some(date) = delivery_date {
            pending_query.push(" AND o.delivery_date = ");
            pending_query.push_bind(date);
        }

        let pending_delivery = pending_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get pending delivery orders count"))?;

        // 合并查询：同时统计配送中订单总数（PREPARING）和已完成配送订单总数（DELIVERED）
        let mut delivery_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT 
                COUNT(DISTINCT CASE WHEN pd.delivery_status = 'PREPARING' THEN o.id END) as delivering_count,
                COUNT(DISTINCT CASE WHEN pd.delivery_status = 'DELIVERED' THEN o.id END) as completed_count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            JOIN provider_deliveries pd ON pd.assignment_id = poa.id
            WHERE t.name_hash = 
            "#,
        );

        delivery_query.push_bind(provider_hash);
        delivery_query.push(" AND t.tenant_type = 'PROVIDER'");
        delivery_query.push(" AND t.deleted_at IS NULL");
        delivery_query.push(" AND o.deleted_at IS NULL");
        delivery_query.push(" AND pd.delivery_status IN ('PREPARING', 'DELIVERED')");

        if let Some(date) = delivery_date {
            delivery_query.push(" AND o.delivery_date = ");
            delivery_query.push_bind(date);
        }

        let delivery_stats = delivery_query
            .build_query_as::<DeliveryStatsRow>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get delivery orders count"))?;

        let delivering = delivery_stats.delivering_count;
        let completed = delivery_stats.completed_count;

        // 查询退换货任务总数（return_exchange_records 表中 status = 'PENDING'）
        let mut return_exchange_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT COUNT(DISTINCT rer.id) as count
            FROM return_exchange_records rer
            JOIN order_details od ON rer.order_detail_id = od.id
            JOIN orders o ON od.order_id = o.id
            JOIN provider_orders_assignments poa ON o.id = poa.order_id
            JOIN tenants t ON poa.provider_id = t.id
            WHERE t.name_hash = 
            "#,
        );

        return_exchange_query.push_bind(provider_hash);
        return_exchange_query.push(" AND t.tenant_type = 'PROVIDER'");
        return_exchange_query.push(" AND t.deleted_at IS NULL");
        return_exchange_query.push(" AND rer.status = 'PENDING'");
        return_exchange_query.push(" AND o.deleted_at IS NULL");

        if let Some(date) = delivery_date {
            return_exchange_query.push(" AND o.delivery_date = ");
            return_exchange_query.push_bind(date);
        }

        let return_exchange_tasks = return_exchange_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get return exchange tasks count"))?;

        // 查询待备货 SKU 总数（订单状态为 ASSIGNED 的订单的 product_code 去重）
        let mut pending_stock_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT COUNT(DISTINCT od.product_code) as count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            JOIN order_details od ON o.id = od.order_id
            WHERE t.name_hash = 
            "#,
        );

        pending_stock_query.push_bind(provider_hash);
        pending_stock_query.push(" AND t.tenant_type = 'PROVIDER'");
        pending_stock_query.push(" AND t.deleted_at IS NULL");
        pending_stock_query.push(" AND o.order_status = 'ASSIGNED'");
        pending_stock_query.push(" AND o.deleted_at IS NULL");

        if let Some(date) = delivery_date {
            pending_stock_query.push(" AND o.delivery_date = ");
            pending_stock_query.push_bind(date);
        }

        let pending_stock_skus = pending_stock_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get pending stock SKUs count"))?;

        // 查询已签收的 SKU 数量（COMPLETED 订单中，CUSTOMER 验收结果为 PASS 或 PARTIAL，且 accepted=1 的 inspected_qty 总和）
        let mut accepted_skus_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT COALESCE(SUM(oii.inspected_qty), 0) as accepted_skus_qty
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            JOIN order_inspections oi ON oi.order_id = o.id
            JOIN order_inspection_items oii ON oii.inspection_id = oi.id
            WHERE t.name_hash = 
            "#,
        );

        accepted_skus_query.push_bind(provider_hash);
        accepted_skus_query.push(" AND t.tenant_type = 'PROVIDER'");
        accepted_skus_query.push(" AND t.deleted_at IS NULL");
        accepted_skus_query.push(" AND o.order_status = 'COMPLETED'");
        accepted_skus_query.push(" AND o.deleted_at IS NULL");
        accepted_skus_query.push(" AND oi.inspection_result IN ('PASS', 'PARTIAL')");
        accepted_skus_query.push(" AND oi.inspected_by_type = 'CUSTOMER'");
        accepted_skus_query.push(" AND oii.accepted = 1");

        if let Some(date) = delivery_date {
            accepted_skus_query.push(" AND o.delivery_date = ");
            accepted_skus_query.push_bind(date);
        }

        let accepted_skus_qty = accepted_skus_query
            .build_query_scalar::<sqlx::types::Decimal>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get accepted SKUs quantity"))?;

        debug!(
            "Dashboard stats - pending_delivery: {}, delivering: {}, completed: {}, return_exchange: {}, pending_stock: {}, accepted_skus_qty: {}",
            pending_delivery, delivering, completed, return_exchange_tasks, pending_stock_skus, accepted_skus_qty
        );

        Ok(ProviderDashboardStatsDTO {
            pending_delivery_orders: pending_delivery,
            delivering_orders: delivering,
            completed_orders: completed,
            return_exchange_tasks: return_exchange_tasks,
            pending_stock_skus: pending_stock_skus,
            accepted_skus_qty: accepted_skus_qty,
        })
    }

    async fn get_provider_today_delivered_products(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<ProviderTodayDeliveredProductsDTO>, AppError> {
        use tracing::debug;

        debug!(
            "Fetching today delivered products for provider_hash: {}",
            provider_hash
        );

        let products = sqlx::query_as::<_, ProviderTodayDeliveredProductsDTO>(
            r#"
            SELECT 
                od.id as order_detail_id,
                od.product_code,
                od.product_name,
                od.category_name,
                od.unit,
                COALESCE(SUM(pdi.actual_qty), 0) as total_actual_qty
            FROM orders o
            JOIN provider_orders_assignments poa ON o.id = poa.order_id
            JOIN tenants t ON poa.provider_id = t.id
            JOIN provider_deliveries pd ON pd.assignment_id = poa.id
            JOIN provider_delivery_items pdi ON pd.id = pdi.delivery_id
            JOIN order_details od ON pdi.order_detail_id = od.id
            WHERE t.name_hash = ?
                AND t.tenant_type = 'PROVIDER'
                AND t.deleted_at IS NULL
                AND o.deleted_at IS NULL
                AND o.delivery_date = CURDATE()
                AND pd.delivery_status = 'DELIVERED'
            GROUP BY od.id, od.product_code, od.product_name, od.category_name, od.unit
            ORDER BY od.product_code
            "#,
        )
        .bind(provider_hash)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!(
            "Failed to get provider today delivered products"
        ))?;

        debug!("Found {} delivered products for today", products.len());
        Ok(products)
    }
}
