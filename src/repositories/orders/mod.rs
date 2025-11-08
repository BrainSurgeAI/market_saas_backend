pub(crate) mod customer;
pub(crate) mod provider;

pub(crate) mod marketplace;
pub(crate) mod shared_market_customer;
pub(crate) mod common;

use crate::{
    common::AppError,
    map_db_err,
    models::{
        order_action::OrderAction, order_machine::OrderStateMachine, order_status::OrderStatus,
        tenant_type::TenantType,
    },
};

use super::my_sql_repository::MySqlRepository;

use tracing::error;

impl MySqlRepository {
    /// Helper method to insert order details in batch
    /// This improves performance by preparing processing requirements once
    /// and using a single method for the detail insertion logic    
    async fn insert_order_details(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
        order_id: u64,
        items: &[crate::dto::order::CreateOrderItem],
    ) -> Result<(), AppError> {
        for item in items {
            // Prepare processing requirements efficiently
            let processing_requirements = if item.processing_services.is_empty() {
                None
            } else {
                Some(
                    item.processing_services
                        .iter()
                        .map(|service| {
                            format!(
                                "{}:{}",
                                service.name,
                                service.description.as_deref().unwrap_or_default()
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(","),
                )
            };

            sqlx::query!(
                r#"INSERT INTO order_details (
                    order_id, product_code, product_name, category_id, category_name,
                    unit, quantity, original_price, discount_rate, actual_price,
                    actual_amount, total_amount, processing_requirements, remark
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                order_id,
                &item.product_code,
                &item.product_name,
                &item.category_id,
                &item.category_name,
                &item.unit,
                &item.quantity,
                &item.original_price,
                &item.discount_rate,
                &item.price,
                &item.total,
                &item.total,
                &processing_requirements,
                &item.remark
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                error!("Failed to create order detail: {:#?}", e);
                AppError::Database(e)
            })?;
        }
        Ok(())
    }

    /// Helper method to validate and transition order state using OrderStateMachine
    /// This method encapsulates the logic for:
    /// 1. Converting current status string to OrderStatus enum
    /// 2. Getting the next valid state based on action and tenant type
    /// 3. Handling state transition errors with custom error messages
    ///
    /// # Arguments
    /// * `current_status_str` - Current order status as string
    /// * `action` - The action to transition from current state
    /// * `tenant_type` - The tenant type performing the action
    /// * `error_msg` - Custom error message if transition fails
    ///
    /// # Returns
    /// The next OrderStatus if transition is valid, or AppError if invalid
    #[allow(unused)]
    fn validate_and_transition_order_state(
        current_status_str: &str,
        action: OrderAction,
        tenant_type: TenantType,
        error_msg: &str,
    ) -> Result<OrderStatus, AppError> {
        let current_status = OrderStatus::try_from(current_status_str)?;

        OrderStateMachine::next_state(current_status, action, tenant_type).map_err(|e| {
            error!("Order state transition error: {}", e);
            AppError::Validation(error_msg.to_string())
        })
    }

    /// Helper method to fetch provider's order information
    /// This method retrieves order details for a provider
    ///
    /// # Arguments
    /// * `provider_hash` - Hash of the provider tenant
    /// * `order_code` - Unique order code
    /// * `error_msg` - Custom error message if order not found
    ///
    /// # Returns
    /// A tuple containing (order_id, order_status)
    async fn fetch_provider_order(
        &self,
        provider_hash: &str,
        order_code: &str,
        error_msg: &str,
    ) -> Result<(i32, String), AppError> {
        let order = sqlx::query!(
            r#"SELECT o.id, o.order_status FROM tenants t
                   INNER JOIN provider_orders_assignments poa ON t.id = poa.provider_id
                   INNER JOIN orders o ON poa.order_id = o.id
                   WHERE t.tenant_type = 'PROVIDER' AND t.name_hash = ? AND o.order_code = ?"#,
            provider_hash,
            order_code
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order"))?
        .ok_or_else(|| {
            error!("Order {} was not found: {}", order_code, error_msg);
            AppError::not_found(error_msg.to_string())
        })?;

        Ok((order.id, order.order_status))
    }
}
