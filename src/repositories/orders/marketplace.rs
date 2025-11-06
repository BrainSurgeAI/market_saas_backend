use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    map_db_err,
    models::{
        order_action::OrderAction, order_machine::OrderStateMachine, order_status::OrderStatus,
        tenant_type::TenantType,
    },
};
use async_trait::async_trait;
use tracing::error;

#[async_trait]
pub(crate) trait MarketplaceOrderRepository: Send + Sync {

    /// Marketplace assign order to provider
    ///
    /// # Arguments
    /// * `order_code` - The code of the order
    /// * `provider_id` - The id of the provider
    /// * `confirmed_by` - The username of the user who confirmed the order
    ///
    /// # Returns
    /// A result containing the error if the order is not found or the order is expired
    /// or the order is not in the correct status to be assigned to a provider
    async fn assign_order(&self, order_code: &str, provider_id: i32, confirmed_by: &str) -> Result<OrderStatus, AppError>;
}

#[async_trait]
impl MarketplaceOrderRepository for MySqlRepository {
    async fn assign_order(&self, order_code: &str, provider_id: i32, confirmed_by: &str) -> Result<OrderStatus, AppError> {
        // Only `PENDING` status of the order can be assigned
        let current = OrderStatus::Pending;
        let order_opt = sqlx::query!(
            r#"SELECT id, delivery_date FROM orders WHERE order_code = ? AND order_status = ?"#,
            order_code,
            current.to_str()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order by id"))?;

        let order =
            order_opt.ok_or_else(|| AppError::NotFound(format!("订单 {} 不存在", order_code)))?;

        if order.delivery_date < chrono::Local::now().date_naive() {
            return Err(AppError::Validation(format!(
                "不能指派已过期订单 {}",
                order_code
            )));
        }

        let order_id = order.id;

        let next = OrderStateMachine::next_state(
            OrderStatus::Pending,
            OrderAction::AssignSupplier,
            TenantType::Market,
        )
        .map_err(|e| {
            error!("{}", e);
            return AppError::Validation("Invalid transition".to_string());
        })?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // update order status to confirmed
        sqlx::query!(
            r#"UPDATE orders SET order_status = ?, confirmed_at = NOW(), confirmed_by = ? WHERE id = ? AND order_status = ?"#,
            &next.to_str(),
            confirmed_by,
            order_id,
            &current.to_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status from PENDING to ASSIGNED"))?;

        // dispatch order to provider
        sqlx::query!(
            r#"INSERT INTO provider_orders_assignments (order_id, provider_id) VALUES (?, ?)"#,
            order_id,
            provider_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert provider orders assignments"))?;

        sqlx::query!(
            r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason ) VALUES (?, ?, ?, ?, ?)"#,
            order_id, &current.to_str(), next.to_str(), confirmed_by, OrderAction::AssignSupplier.description()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert order status history from Pending to Assigned"))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(next)
    }
}
