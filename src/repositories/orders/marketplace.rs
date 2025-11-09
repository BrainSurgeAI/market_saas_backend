use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    map_db_err,
    models::claims::Claims,
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
    async fn assign_order(
        &self,
        order_code: &str,
        provider_id: i32,
        confirmed_by: &str,
    ) -> Result<OrderStatus, AppError>;

    async fn deliver_to_customer(
        &self,
        order_code: &str,
        claims: &Claims,
        action: OrderAction,
    ) -> Result<OrderStatus, AppError>;
}

#[async_trait]
impl MarketplaceOrderRepository for MySqlRepository {
    async fn assign_order(
        &self,
        order_code: &str,
        provider_id: i32,
        confirmed_by: &str,
    ) -> Result<OrderStatus, AppError> {
        use chrono::Local;
        let current = OrderStatus::Pending;

        // begin transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction to assign order"))?;

        let order_opt = sqlx::query!(
            r#"
            SELECT id, delivery_date, order_status
            FROM orders
            WHERE order_code = ?
            FOR UPDATE
            "#,
            order_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to lock order for update"))?;

        let order =
            order_opt.ok_or_else(|| AppError::NotFound(format!("订单 {} 不存在", order_code)))?;

        if order.order_status != current.to_str() {
            return Err(AppError::Validation(format!(
                "订单 {} 状态不是可分配状态 (当前: {})",
                order_code, order.order_status
            )));
        }

        if order.delivery_date < Local::now().date_naive() {
            return Err(AppError::Validation(format!(
                "不能指派已过期订单 {}",
                order_code
            )));
        }

        let next =
            OrderStateMachine::next_state(current, OrderAction::AssignSupplier, TenantType::Market)
                .map_err(|e| {
                    error!("状态流转错误: {}", e);
                    AppError::Validation("Invalid transition".to_string())
                })?;

        let affected = sqlx::query!(
            r#"
            UPDATE orders
            SET order_status = ?, confirmed_at = NOW(), confirmed_by = ?
            WHERE id = ? AND order_status = ?
            "#,
            next.to_str(),
            confirmed_by,
            order.id,
            current.to_str()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::Validation(format!(
                "订单 {} 状态已更新或被取消",
                order_code
            )));
        }

        sqlx::query!(
            r#"INSERT INTO provider_orders_assignments (order_id, provider_id)
               VALUES (?, ?)"#,
            order.id,
            provider_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert provider orders assignments"))?;

        sqlx::query!(
            r#"
            INSERT INTO order_status_history 
                (order_id, from_status, to_status, changed_by, change_reason)
            VALUES (?, ?, ?, ?, ?)
            "#,
            order.id,
            current.to_str(),
            next.to_str(),
            confirmed_by,
            OrderAction::AssignSupplier.description()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert order status history"))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        Ok(next)
    }

    async fn deliver_to_customer(
        &self,
        order_code: &str,
        claims: &Claims,
        action: OrderAction,
    ) -> Result<OrderStatus, AppError> {
        let mut tx = self.pool.begin().await.map_err(map_db_err!(
            "Failed to begin transaction to deliver to customer"
        ))?;

        let order_opt = sqlx::query!(
            r#"
            SELECT o.id, o.order_status 
            FROM orders o 
            INNER JOIN tenants t ON t.id = o.market_id 
            WHERE o.order_code = ? AND t.name_hash = ? AND t.tenant_type = ? FOR UPDATE"#,
            order_code,
            claims.tenant_hash,
            claims.tenant_type
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to lock order for update"))?;

        let order =
            order_opt.ok_or_else(|| AppError::NotFound(format!("订单 {} 不存在", order_code)))?;


        let next = OrderStateMachine::next_state(OrderStatus::try_from(order.order_status.as_str())?, action, TenantType::try_from(claims.tenant_type.as_str())?)
            .map_err(|e| {
                error!("状态流转错误: {}", e);
                AppError::Validation("Invalid transition".to_string())
            })?;

        sqlx::query!(
            r#"UPDATE orders SET order_status = ? WHERE id = ?"#,
            next.to_str(),
            order.id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to update order status"))?;

        sqlx::query!(
            r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason) VALUES (?, ?, ?, ?, ?)"#,
            order.id, order.order_status, next.to_str(), claims.real_name, action.description()
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to insert order status history"))?;

        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        Ok(next)
    }
}
