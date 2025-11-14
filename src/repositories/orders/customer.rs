use crate::{
    common::AppError,
    dto::order::CreateOrderRequestDTO,
    // 以下导入已注释，如果将来需要查询分类层级映射时，可以取消注释
    // dto::category::CategoryLevel1Row,
    map_db_err,
    models::{claims::Claims, order_action::OrderAction, order_status::OrderStatus},
    repositories::{generate_code, my_sql_repository::MySqlRepository, CodeType},
};

use async_trait::async_trait;
use futures::future::try_join_all;
use sqlx::QueryBuilder;

use rust_decimal::Decimal;
use tracing::info;

#[async_trait]
pub(crate) trait CustomerOrderRepository: Send + Sync {
    /// Create a new order by a customer
    ///
    /// # Arguments
    /// * `customer_hash` - The hash of the customer
    /// * `order` - The order to create with order items information
    ///
    /// # Returns
    /// A result containing the order response if the order is created successfully
    /// or the error if the order is not created
    async fn create_order(
        &self,
        claims: &Claims,
        order_payload: &CreateOrderRequestDTO,
    ) -> Result<String, AppError>;
}

#[async_trait]
impl CustomerOrderRepository for MySqlRepository {
    async fn create_order(
        &self,
        claims: &Claims,
        order_payload: &CreateOrderRequestDTO,
    ) -> Result<String, AppError> {
        if order_payload.items.is_empty() {
            return Err(AppError::bad_request("订单不能为空"));
        }

        let record = sqlx::query!(
            r#"SELECT t.id AS customer_id, t.name AS customer_name, tr.market_id 
               FROM tenants t 
               INNER JOIN tenant_relationships tr ON tr.provider_id = t.id 
               WHERE t.name_hash = ? AND tr.status = 'ACTIVE' AND t.tenant_type = ?"#,
            claims.tenant_hash,
            claims.tenant_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get customer information"))?
        .ok_or_else(|| AppError::not_found("Customer not found"))?;

        // Generate order code before transaction to ensure consistency
        let order_code = generate_code(CodeType::Order);

        // 计算每个商品的折扣价格和金额，同时获取产品信息
        let item_data_futures: Vec<_> = order_payload
            .items
            .iter()
            .map(|item| {
                let product_code = item.product_code.clone();
                let market_id = record.market_id;
                let category_level_1_id = item.category_level_1_id;
                let customer_id = record.customer_id;
                let ordered_qty = item.ordered_qty;
                let processing_services = item.processing_services.clone();
                let remark = item.remark.clone();
                let repo = self.clone();

                async move {
                    // 获取价格和折扣率
                    let (unit_price, discount_rate) = repo
                        .get_product_price_with_discount_rate(
                            &product_code,
                            market_id,
                            category_level_1_id,
                            customer_id,
                        )
                        .await?;

                    // 获取产品信息（product_name, unit, category_name）
                    // category_level_1_id 就是一级分类ID，直接查询一级分类名称
                    let product_info = sqlx::query!(
                        r#"
                        SELECT p.name as product_name, p.unit, c.name as category_name
                        FROM products p
                        JOIN categories c ON c.id = ? AND c.level = 1
                        WHERE p.product_code = ?
                        "#,
                        category_level_1_id,
                        product_code
                    )
                    .fetch_optional(&repo.pool)
                    .await
                    .map_err(map_db_err!("Failed to get product info"))?
                    .ok_or_else(|| {
                        AppError::NotFound(format!("Product info not found: {}", product_code))
                    })?;

                    let ordered_amount = unit_price * ordered_qty;
                    let discounted_unit_price = unit_price * discount_rate;

                    // 处理加工要求
                    let processing_requirements = if processing_services.is_empty() {
                        None
                    } else {
                        Some(
                            processing_services
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

                    Ok::<_, AppError>((
                        product_code,
                        product_info.product_name,
                        category_level_1_id,
                        product_info.category_name,
                        product_info.unit,
                        ordered_qty,
                        unit_price,
                        discount_rate,
                        discounted_unit_price,
                        processing_requirements,
                        remark,
                        ordered_amount
                    ))
                }
            })
            .collect();

        let item_data = try_join_all(item_data_futures).await?;

        // 计算总金额
        let ordered_amount = item_data.iter().fold(
            Decimal::ZERO,
            |acc_ordered, (_, _, _, _, _, _, _, _, _, _, _, ordered_amount)| {
                acc_ordered + ordered_amount
            },
        );

        // Use transaction for atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // Insert order and get ID
        let order_id = sqlx::query!(
            r#"INSERT INTO orders (
                order_code, market_id, customer_id, ordered_amount,
                delivery_date, delivery_address, contact_name, contact_phone, created_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            order_code,
            record.market_id,
            record.customer_id,
            ordered_amount,  
            order_payload.delivery_info.delivery_date,
            order_payload.delivery_info.delivery_address,
            order_payload.delivery_info.contact_name,
            order_payload.delivery_info.contact_phone,
            claims.real_name
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to create order"))?
        .last_insert_id();

        // Batch insert order details for better performance
        if !item_data.is_empty() {
            let mut query_builder = QueryBuilder::new(
                r#"INSERT INTO order_details (
                    order_id, product_code, product_name, category_id, category_name,
                    unit, ordered_qty, unit_price, discount_rate, discounted_unit_price,
                    ordered_amount, processing_requirements, remark
                )"#,
            );

            query_builder.push_values(item_data.iter(), |mut b, item| {
                let (
                    product_code,
                    product_name,
                    category_id,
                    category_name,
                    unit,
                    ordered_qty,
                    unit_price,
                    discount_rate,
                    discounted_unit_price,
                    processing_requirements,
                    remark,
                    ordered_amount,
                ) = item;
                b.push_bind(order_id as i32)
                    .push_bind(product_code)
                    .push_bind(product_name)
                    .push_bind(category_id)
                    .push_bind(category_name)
                    .push_bind(unit)
                    .push_bind(ordered_qty)
                    .push_bind(unit_price)
                    .push_bind(discount_rate)
                    .push_bind(discounted_unit_price)
                    .push_bind(ordered_amount)
                    .push_bind(processing_requirements.as_ref())
                    .push_bind(remark.as_ref());
            });

            query_builder
                .build()
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to batch insert order details"))?;
        }

        // create a record in order_status_history table
        self.insert_order_status_history(
            &mut tx,
            order_id as i32,
            "CREATED",
            OrderStatus::Pending,
            &claims.real_name.as_str(),
            OrderAction::Create,
        )
        .await?;

        // Commit transaction
        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        info!("Order {} created successfully", order_code);

        Ok(order_code)
    }
}
