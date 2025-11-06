use crate::{
    common::AppError,
    dto::order::{CreateOrderDTO, OrderResponse},
    models::order_status::OrderStatus,
    map_db_err,
    repositories::{generate_code, CodeType},
    repositories::my_sql_repository::MySqlRepository
};

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use tracing::{error, info};

#[async_trait]
pub(crate) trait CustomerOrderRepository: Send + Sync {
    /// Create a new order for a customer
    ///
    /// # Arguments
    /// * `customer_hash` - The hash of the customer
    /// * `order` - The order to create
    ///
    /// # Returns
    /// A result containing the order response if the order is created successfully
    /// or the error if the order is not created
    async fn create_order(
        &self,
        customer_hash: &str,
        order: &CreateOrderDTO,
    ) -> Result<OrderResponse, AppError>;
}

#[async_trait]
impl CustomerOrderRepository for MySqlRepository {
    async fn create_order(
        &self,
        customer_hash: &str,
        order: &CreateOrderDTO,
    ) -> Result<OrderResponse, AppError> {
         // Fetch customer information with early return for better error handling
         let record = sqlx::query!(
            r#"SELECT t.id as customer_id, t.name as customer_name, tr.market_id from tenants t 
               INNER JOIN tenant_relationships tr ON tr.provider_id = t.id 
               WHERE t.name_hash = ? AND tr.status = 'ACTIVE'"#,
            customer_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get customer information"))?
        .ok_or_else(|| AppError::NotFound("Customer not found".to_string()))?;

        // Pre-calculate order amounts for efficiency
        let total_amount: Decimal = order.items.iter().map(|item| item.original_amount).sum();

        let discount_amount: Decimal = order
            .items
            .iter()
            .map(|item| item.original_amount - item.total)
            .sum();

        // Parse delivery date once, with proper error handling
        let delivery_date =
            NaiveDate::parse_from_str(&order.delivery_info.delivery_date, "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());

        // Generate order code before transaction to ensure consistency
        let order_code = generate_code(CodeType::Order);

        // Use transaction for atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // Insert order and get ID
        let order_id = sqlx::query!(
            r#"INSERT INTO orders (
                order_code, market_id, customer_id, total_amount, discount_amount,
                actual_amount, delivery_date, delivery_address, contact_name,
                contact_phone, created_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            order_code,
            record.market_id,
            record.customer_id,
            total_amount,
            discount_amount,
            order.total_amount, // Frontend total_amount is actual quantity x discounted price
            order.delivery_info.delivery_date,
            order.delivery_info.delivery_address,
            order.delivery_info.contact_name,
            order.delivery_info.contact_phone,
            record.customer_name
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to create order"))?
        .last_insert_id();

        // Batch insert order details for better performance
        self.insert_order_details(&mut tx, order_id, &order.items)
            .await
            .map_err(|e| {
                error!("Failed to create order details: {:#?}", e);
                AppError::Internal(e.to_string())
            })?;

        // Commit transaction
        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        info!("Order {} created successfully", order_code);

        Ok(OrderResponse {
            order_code,
            total_amount,
            actual_amount: order.total_amount,
            delivery_address: order.delivery_info.delivery_address.clone(),
            delivery_date,
            order_status: OrderStatus::Pending.to_string(),
            created_at: None,
            after_sale_at: None,
        })
    }
}