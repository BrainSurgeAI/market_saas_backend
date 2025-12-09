use crate::{
    common::AppError,
    dto::order::{CreateOrderRequestDTO, CustomerStatisticsDTO, TopProductDTO},
    // 以下导入已注释，如果将来需要查询分类层级映射时，可以取消注释
    // dto::category::CategoryLevel1Row,
    map_db_err,
    models::{claims::Claims, order_action::OrderAction, order_status::OrderStatus},
    repositories::{generate_code, my_sql_repository::MySqlRepository, CodeType},
};

use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate};
use futures::future::try_join_all;
use sqlx::QueryBuilder;

use rust_decimal::Decimal;
use tracing::info;

/// Month range structure containing start and end dates
struct MonthRange {
    start: NaiveDate,
    end: NaiveDate,
}

/// Calculate current month and previous month date ranges
///
/// # Returns
/// A tuple containing (current_month_range, previous_month_range)
fn calculate_month_ranges() -> Result<(MonthRange, MonthRange), AppError> {
    let now = Local::now();
    let current_date = now.date_naive();
    
    // Calculate current month start and end
    let current_month_start = NaiveDate::from_ymd_opt(
        current_date.year(),
        current_date.month(),
        1,
    ).ok_or_else(|| AppError::Internal("Failed to calculate current month start".to_string()))?;
    
    let current_month_end = if current_date.month() == 12 {
        NaiveDate::from_ymd_opt(
            current_date.year() + 1,
            1,
            1,
        ).ok_or_else(|| AppError::Internal("Failed to calculate current month end".to_string()))?
            .pred_opt()
            .ok_or_else(|| AppError::Internal("Failed to calculate current month end".to_string()))?
    } else {
        NaiveDate::from_ymd_opt(
            current_date.year(),
            current_date.month() + 1,
            1,
        ).ok_or_else(|| AppError::Internal("Failed to calculate current month end".to_string()))?
            .pred_opt()
            .ok_or_else(|| AppError::Internal("Failed to calculate current month end".to_string()))?
    };

    // Calculate previous month start and end
    let previous_month_start = if current_date.month() == 1 {
        NaiveDate::from_ymd_opt(
            current_date.year() - 1,
            12,
            1,
        ).ok_or_else(|| AppError::Internal("Failed to calculate previous month start".to_string()))?
    } else {
        NaiveDate::from_ymd_opt(
            current_date.year(),
            current_date.month() - 1,
            1,
        ).ok_or_else(|| AppError::Internal("Failed to calculate previous month start".to_string()))?
    };

    let previous_month_end = current_month_start
        .pred_opt()
        .ok_or_else(|| AppError::Internal("Failed to calculate previous month end".to_string()))?;

    Ok((
        MonthRange {
            start: current_month_start,
            end: current_month_end,
        },
        MonthRange {
            start: previous_month_start,
            end: previous_month_end,
        },
    ))
}

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

    /// Get customer statistics including orders, spending, return rate and trends
    ///
    /// # Arguments
    /// * `claims` - The claims containing customer information
    ///
    /// # Returns
    /// Customer statistics including orders count, total spent, return rate,
    /// trends compared to previous month, and top products
    async fn get_customer_statistics(
        &self,
        claims: &Claims,
    ) -> Result<CustomerStatisticsDTO, AppError>;
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

    async fn get_customer_statistics(
        &self,
        claims: &Claims,
    ) -> Result<CustomerStatisticsDTO, AppError> {
        // Get customer_id from tenant_hash
        let record = sqlx::query!(
            r#"SELECT t.id AS customer_id 
               FROM tenants t 
               WHERE t.name_hash = ? AND t.tenant_type = ? AND t.deleted_at IS NULL"#,
            claims.tenant_hash,
            claims.tenant_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get customer information"))?
        .ok_or_else(|| AppError::not_found("Customer not found"))?;

        // Calculate current month and previous month date ranges
        let (current_month, previous_month) = calculate_month_ranges()?;

        // Query current month statistics
        let current_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT o.id) as orders_count,
                COALESCE(SUM(o.net_amount), 0) as total_spent
            FROM orders o
            WHERE o.customer_id = ?
              AND DATE(o.created_at) >= ?
              AND DATE(o.created_at) <= ?
              AND o.deleted_at IS NULL
            "#,
            record.customer_id,
            current_month.start,
            current_month.end
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get current month statistics"))?;

        // Query current month return count from return_exchange_records
        let current_return_count = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT rer.id) as return_count
            FROM return_exchange_records rer
            INNER JOIN order_details od ON rer.order_detail_id = od.id
            INNER JOIN orders o ON od.order_id = o.id
            WHERE o.customer_id = ?
              AND DATE(o.created_at) >= ?
              AND DATE(o.created_at) <= ?
              AND rer.operation_type = 'RETURN' OR rer.operation_type = 'EXCHANGE'
              AND o.deleted_at IS NULL
            "#,
            record.customer_id,
            current_month.start,
            current_month.end
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get current month return count"))?;

        // Query previous month statistics
        let previous_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT o.id) as orders_count,
                COALESCE(SUM(o.net_amount), 0) as total_spent
            FROM orders o
            WHERE o.customer_id = ?
              AND DATE(o.created_at) >= ?
              AND DATE(o.created_at) <= ?
              AND o.deleted_at IS NULL
            "#,
            record.customer_id,
            previous_month.start,
            previous_month.end
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get previous month statistics"))?;

        // Query previous month return count from return_exchange_records
        let previous_return_count = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT rer.id) as return_count
            FROM return_exchange_records rer
            INNER JOIN order_details od ON rer.order_detail_id = od.id
            INNER JOIN orders o ON od.order_id = o.id
            WHERE o.customer_id = ?
              AND DATE(o.created_at) >= ?
              AND DATE(o.created_at) <= ?
              AND rer.operation_type = 'RETURN' OR rer.operation_type = 'EXCHANGE'
              AND o.deleted_at IS NULL
            "#,
            record.customer_id,
            previous_month.start,
            previous_month.end
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get previous month return count"))?;

        let current_orders = current_stats.orders_count as i64;
        let previous_orders = previous_stats.orders_count as i64;
        let current_total_spent = current_stats.total_spent;
        let previous_total_spent = previous_stats.total_spent;
        let current_return_count = current_return_count.return_count as i64;
        let previous_return_count = previous_return_count.return_count as i64;

        // Calculate return rate
        let current_return_rate = if current_orders > 0 {
            (current_return_count as f64 / current_orders as f64) * 100.0
        } else {
            0.0
        };

        let previous_return_rate = if previous_orders > 0 {
            (previous_return_count as f64 / previous_orders as f64) * 100.0
        } else {
            0.0
        };

        // Calculate trends
        let orders_trend = if previous_orders > 0 {
            let change = ((current_orders as f64 - previous_orders as f64) / previous_orders as f64) * 100.0;
            format!("{}{:.0}%", if change >= 0.0 { "+" } else { "" }, change)
        } else {
            if current_orders > 0 {
                "+100%".to_string()
            } else {
                "0%".to_string()
            }
        };
        let orders_trend_up = current_orders >= previous_orders;

        let total_spent_trend = if previous_total_spent > Decimal::ZERO {
            let change = ((current_total_spent - previous_total_spent) / previous_total_spent) * Decimal::from(100);
            format!("{}{:.0}%", if change >= Decimal::ZERO { "+" } else { "" }, change)
        } else {
            if current_total_spent > Decimal::ZERO {
                "+100%".to_string()
            } else {
                "0%".to_string()
            }
        };
        let total_spent_trend_up = current_total_spent >= previous_total_spent;

        let return_rate_trend = if previous_return_rate > 0.0 {
            let change = current_return_rate - previous_return_rate;
            format!("{}{:.1}%", if change >= 0.0 { "+" } else { "" }, change)
        } else {
            if current_return_rate > 0.0 {
                "+100%".to_string()
            } else {
                "0%".to_string()
            }
        };
        let return_rate_trend_up = current_return_rate >= previous_return_rate; // Lower is better

        // Query top products
        let top_products = sqlx::query!(
            r#"
            SELECT 
                od.product_name as name,
                COUNT(*) as count
            FROM order_details od
            INNER JOIN orders o ON od.order_id = o.id
            WHERE o.customer_id = ?
              AND DATE(o.created_at) >= ?
              AND DATE(o.created_at) <= ?
              AND o.deleted_at IS NULL
            GROUP BY od.product_name
            ORDER BY count DESC
            LIMIT 4
            "#,
            record.customer_id,
            current_month.start,
            current_month.end
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get top products"))?
        .into_iter()
        .map(|row| TopProductDTO {
            name: row.name,
            count: row.count as i64,
        })
        .collect();

        Ok(CustomerStatisticsDTO {
            orders: current_orders,
            total_spent: current_total_spent,
            return_rate: (current_return_rate * 10.0).round() / 10.0, // Round to 1 decimal place
            orders_trend,
            orders_trend_up,
            total_spent_trend,
            total_spent_trend_up,
            return_rate_trend,
            return_rate_trend_up,
            top_products,
        })
    }
}