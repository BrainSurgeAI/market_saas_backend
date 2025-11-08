use crate::{
    common::AppError,
    dto::order::{DeliverToMarketDTO, ExchangeDTO, ExchangeItemUpdateDTO},
    map_db_err,
    models::{
        order_action::OrderAction, order_machine::OrderStateMachine, order_status::OrderStatus,
        tenant_type::TenantType,
    },
    repositories::my_sql_repository::MySqlRepository,
};
use async_trait::async_trait;
use tracing::{debug, error};

#[async_trait]
pub(crate) trait ProviderOrderRepository: Send + Sync {
    async fn order_start_progress(
        &self,
        order_code: &str,
        operator: &str,
        provider_hash: &str,
        delivery_staff_id: Option<&str>,
    ) -> Result<OrderStatus, AppError>;

    async fn deliver_to_market(
        &self,
        order_code: &str,
        provider_hash: &str,
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
    /// let result = repository.update_exchange_item_actual_quantity("operator", &exchange_item_update_dto);
    /// ```
    async fn update_exchange_item_actual_quantity(
        &self,
        operator: &str,
        exchange_item_update_dto: &ExchangeItemUpdateDTO,
    ) -> Result<(), AppError>;
}

#[async_trait]
impl ProviderOrderRepository for MySqlRepository {
    async fn order_start_progress(
        &self,
        order_code: &str,
        operator: &str,
        provider_hash: &str,
        delivery_staff_id: Option<&str>,
    ) -> Result<OrderStatus, AppError> {
        let (order_id, order_status_str) = self
            .fetch_provider_order(
                provider_hash,
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

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        if current_status == OrderStatus::Assigned {
            let id_card = delivery_staff_id
                .ok_or_else(|| AppError::Validation("配送员 ID 不能为空".to_string()))?;

            let delivery_staff = sqlx::query!(
                r#"SELECT id FROM delivery_staff WHERE id_card = ? AND status = true"#,
                id_card
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to get delivery staff by id"))?
            .ok_or_else(|| AppError::NotFound(format!("配送员 {} 没有找到", id_card)))?;

            let update_result = sqlx::query!(
                r#"UPDATE orders
                       SET order_status = ?, delivery_staff_id = ?
                       WHERE id = ? AND order_status = ?"#,
                next.to_str(),
                delivery_staff.id,
                order_id,
                &order_status_str
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!(
                "Failed to update order status from Assigned to SupplierPreparing"
            ))?;

            if update_result.rows_affected() == 0 {
                let _ = tx.rollback().await;
                return Err(AppError::Conflict(format!(
                    "订单 {} 状态已变更，请刷新后重试",
                    order_code
                )));
            }
        } else {
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
        }

        sqlx::query!(
            r#"INSERT INTO order_status_history
                    (order_id, from_status, to_status, changed_by, change_reason)
                  VALUES (?, ?, ?, ?, ?)"#,
            order_id,
            order_status_str,
            next.to_str(),
            operator,
            action.description()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!(
            "Failed to insert order status history from Assigned to SupplierPreparing"
        ))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        Ok(next)
    }

    /// 正常配送订单到市场，非换货订单
    async fn deliver_to_market(
        &self,
        order_code: &str,
        provider_hash: &str,
        deliver_to_market_dto: &DeliverToMarketDTO,
    ) -> Result<(), AppError> {
        // Use the shared helper method to fetch provider's order
        let (order_id, order_status_str) = self
            .fetch_provider_order(
                provider_hash,
                order_code,
                &format!("订单 {} 没有找到", order_code),
            )
            .await?;

        let current_status = OrderStatus::try_from(order_status_str.as_str())?;
        let invalid_state_msg = format!("订单 {} 状态错误，不能配送到市场", order_code);

        let action = match current_status {
            OrderStatus::SupplierPreparing => OrderAction::DeliverToMarket,
            _ => return Err(AppError::Validation(invalid_state_msg)),
        };

        let next = OrderStateMachine::next_state(current_status, action, TenantType::Provider)
            .map_err(|err| {
                error!("{err}");
                AppError::Validation(format!("订单 {} 状态错误，不能配送到市场", order_code))
            })?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction to update order status from SupplierPreparing to SupplierDelivering"))?;

        // update actual quantity and actual amount in order_details table
        for item in deliver_to_market_dto.items.iter() {
            let id = item.id;
            let actual_quantity = item.actual_quantity;

            sqlx::query!(
                r#"UPDATE order_details SET actual_quantity = ?, actual_amount = actual_price * ?,
                   total_amount = original_price * ? WHERE order_id = ? AND id = ?"#,
                actual_quantity,
                actual_quantity,
                actual_quantity,
                order_id,
                id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to update actual quantity"))?;
        }

        // update order status and amount
        sqlx::query!(
            r#"UPDATE orders o
               SET o.actual_amount = (
                   SELECT SUM(od.actual_amount)
                   FROM order_details od
                   WHERE od.order_id = o.id
               ),
               o.total_amount = (
                   SELECT SUM(od.total_amount)
                   FROM order_details od
                   WHERE od.order_id = o.id
               ),
               o.discount_amount = (
                   SELECT SUM(od.total_amount - od.actual_amount)
                   FROM order_details od
                   WHERE od.order_id = o.id
               ),
               o.stocked_at = NOW(),
               o.order_status = ?,
               o.stocked_by = ? 
               WHERE o.id = ?"#,
            next.to_str(),
            deliver_to_market_dto.stocked_by,
            order_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!(
            "Failed to update order amount and status from SupplierPreparing to SupplierDelivering"
        ))?;

        sqlx::query!(
            r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason ) VALUES (?, ?, ?, ?, ?)"#,
            order_id, &order_status_str, next.to_str(), deliver_to_market_dto.stocked_by, action.description()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert order status history from SupplierPreparing to SupplierDelivering"))?;

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
        let (order_id, order_status_str) = self
            .fetch_provider_order(
                provider_hash,
                order_code,
                &format!("订单 {} 没有找到", order_code),
            )
            .await?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;


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

        for item in exchange_dto.items.iter() {
            let id = item.id;
            let actual_quantity = item.actual_quantity;
            //let requested_quantity = item.requested_quantity;

            sqlx::query!(
                    r#"UPDATE order_details SET actual_quantity = receipt_quantity + ?, actual_amount = actual_price * ( receipt_quantity + ? ),
                       total_amount = original_price * ( receipt_quantity + ? ) WHERE order_id = ? AND id = ?"#,
                    actual_quantity, actual_quantity, actual_quantity, order_id, id
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to update actual quantity"))?;
        }

        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ?"#,
            next.to_str(),
            order_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        sqlx::query!(
                r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason ) VALUES (?, ?, ?, ?, ?)"#,
                order_id, &order_status_str, next.to_str(), operator, OrderAction::DeliverToMarket.description()
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to insert order status history"))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(())
    }


    async fn update_exchange_item_actual_quantity(
        &self,
        operator: &str,
        exchange_item_update_dto: &ExchangeItemUpdateDTO,
    ) -> Result<(), AppError> {

        debug!(
            "Provider {} updates exchange item actual quantity for order detail id {} and actual quantity {}",
            operator, exchange_item_update_dto.order_detail_id, exchange_item_update_dto.actual_quantity
        );
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        let exchange_item_id = sqlx::query!(
            r#"SELECT id FROM return_exchange_records WHERE order_detail_id = ? FOR UPDATE"#,
            exchange_item_update_dto.order_detail_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get return exchange record"))?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Return exchange record not found: {}",
                exchange_item_update_dto.order_detail_id
            ))
        })?;

        sqlx::query!(
            r#"UPDATE return_exchange_records SET actual_quantity = ?, status = 'PROGRESS', processed_by = ?, processed_at = NOW()
             WHERE order_detail_id = ?"#,
            exchange_item_update_dto.actual_quantity, operator, exchange_item_update_dto.order_detail_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update return exchange record"))?;

        sqlx::query!(
            r#"UPDATE exchange_items SET shipped_at = NOW(), status = 'PROGRESS'
             WHERE return_exchange_id = ?"#,
            exchange_item_id.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update exchange item status"))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(())
    }
}
