pub(crate) mod customer;
pub(crate) mod provider;

pub(crate) mod common;
pub(crate) mod marketplace;
pub(crate) mod shared_market_customer;

use crate::{
    common::AppError,
    map_db_err,
    models::{
        claims::Claims, order_action::OrderAction, order_machine::OrderStateMachine,
        order_status::OrderStatus, tenant_type::TenantType,
    },
};
use std::collections::HashMap;

use super::my_sql_repository::MySqlRepository;

use rust_decimal::Decimal;
use tracing::{debug, error};

impl MySqlRepository {
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
    /// A tuple containing (order_id, order_status, assignment_id)
    async fn fetch_provider_order(
        &self,
        provider_hash: &str,
        order_code: &str,
        error_msg: &str,
    ) -> Result<(i32, String, u32, Option<u64>, Option<i32>), AppError> {
        debug!(
            "Fetching provider order for order code {} and provider hash {}",
            order_code, provider_hash
        );
        let order = sqlx::query!(
            r#"SELECT o.id, o.order_status, poa.id as assignment_id, pd.id as delivery_id, pd.delivery_round FROM tenants t
                   INNER JOIN provider_orders_assignments poa ON t.id = poa.provider_id
                   INNER JOIN orders o ON poa.order_id = o.id
                   LEFT JOIN provider_deliveries pd ON poa.id = pd.assignment_id
                   WHERE t.tenant_type = 'PROVIDER' AND t.name_hash = ? AND o.order_code = ? 
                   ORDER BY pd.delivery_round DESC LIMIT 1
                   FOR UPDATE"#,
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

        Ok((
            order.id,
            order.order_status,
            order.assignment_id,
            order.delivery_id,
            order.delivery_round,
        ))
    }

    // help function to get tenant id by tenant hash and tenant type
    async fn get_tenant_id_by_tenant_hash_and_tenant_type(
        &self,
        tenant_hash: &str,
        tenant_type: &str,
    ) -> Result<i32, AppError> {
        let tenant = sqlx::query!(
            r#"SELECT id FROM tenants WHERE name_hash = ? AND tenant_type = ? AND status = 'ACTIVE'"#,
            tenant_hash, tenant_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get tenant by id"))?
        .ok_or_else(|| {
            return AppError::NotFound(format!(
                "Can not find Tenant by type {} name_hash {} ",
                tenant_type, tenant_hash
            ));
        })?;

        Ok(tenant.id)
    }

    /// 获取产品的折扣价格
    ///
    /// 根据产品代码、市场ID、分类ID和客户ID，查询产品中间价和客户分类折扣率，
    /// 计算并返回客户下单时该产品的折扣价
    ///
    /// # 参数
    /// * `product_code` - 产品代码
    /// * `market_id` - 市场ID
    /// * `category_level_1_id` - 一级分类ID
    /// * `customer_id` - 客户ID（tenant_id）
    ///
    /// # 返回
    /// 返回计算后的折扣价格，如果产品价格不存在则返回错误
    async fn get_product_price_with_discount_rate(
        &self,
        product_code: &str,
        market_id: i32,
        category_level_1_id: i32,
        customer_id: i32,
    ) -> Result<(Decimal, Decimal), AppError> {
        // 1. 根据 product_code 查询 products 表获取 product_id
        let product = sqlx::query!(
            r#"SELECT id FROM products WHERE product_code = ? AND is_disabled = 0"#,
            product_code
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get product by product_code"))?
        .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_code)))?;

        // 2. 查询 product_prices 表获取中间价（avg_price）
        // 优先查询当天的价格，如果没有则查询最近的历史价格，且状态必须为 PUBLISHED
        let price_result = sqlx::query!(
            r#"
            SELECT COALESCE(today_price.avg_price, history_price.avg_price) as avg_price
            FROM products p
            LEFT JOIN (
                SELECT product_id, avg_price 
                FROM product_prices 
                WHERE price_date = CURDATE() 
                  AND status = 'PUBLISHED'
                  AND market_id = ?
            ) today_price ON p.id = today_price.product_id
            LEFT JOIN (
                SELECT pp.product_id, pp.avg_price 
                FROM product_prices pp
                INNER JOIN (
                    SELECT product_id, MAX(price_date) as latest_date
                    FROM product_prices
                    WHERE status = 'PUBLISHED' 
                      AND price_date < CURDATE()
                      AND market_id = ?
                    GROUP BY product_id
                ) latest ON pp.product_id = latest.product_id 
                          AND pp.price_date = latest.latest_date
                          AND pp.status = 'PUBLISHED'
                          AND pp.market_id = ?
            ) history_price ON p.id = history_price.product_id AND today_price.product_id IS NULL
            WHERE p.id = ?
            "#,
            market_id,
            market_id,
            market_id,
            product.id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get product price"))?
        .and_then(|r| r.avg_price)
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Product price not found for product_code: {}, market_id: {}",
                product_code, market_id
            ))
        })?;

        let avg_price = price_result;

        // 3. 查询 customer_category_discounts 表获取折扣率
        // 根据 tenant_id (customer_id), category_id (category_level_1_id),
        // 当前日期在 start_date 和 end_date 之间，status = true
        let discount_result = sqlx::query!(
            r#"
            SELECT discount_rate 
            FROM customer_category_discounts 
            WHERE tenant_id = ? 
              AND category_id = ? 
              AND status = true
              AND CURRENT_DATE BETWEEN start_date AND COALESCE(end_date, '9999-12-31')
            ORDER BY start_date DESC
            LIMIT 1
            "#,
            customer_id,
            category_level_1_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get customer category discount"))?;

        // 如果没有折扣，则折扣率为 1.0（不打折）
        // discount_rate 在数据库中是 0.8 表示 8 折（原价的 80%），1.0 表示不打折
        let discount_rate = discount_result
            .map(|r| r.discount_rate)
            .unwrap_or(Decimal::new(100, 2)); // 默认 1.00

        Ok((avg_price, discount_rate))
    }

    /// help function to insert into order_status_history table
    ///
    /// # Arguments
    /// * `tx` - The transaction to insert the order status history
    /// * `order_id` i32 - The id of the order
    /// * `from_status` &str - The from status of the order
    /// * `to_status` OrderStatus - The to status of the order
    /// * `changed_by` &str - The changed by of the order
    /// * `action` OrderAction - The action of the order
    ///
    /// # Returns
    /// A result containing the error if the order status history is not inserted
    async fn insert_order_status_history(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
        order_id: i32,
        from_status: &str,
        to_status: OrderStatus,
        changed_by: &str,
        action: OrderAction,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason) VALUES (?, ?, ?, ?, ?)"#,
            order_id, from_status, to_status.to_str(), changed_by, action.description()
        )
            .execute(&mut **tx)
            .await
            .map_err(map_db_err!("Failed to insert order status history"))?;
        Ok(())
    }

    async fn create_order_inspection_record(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
        order_id: i32,
        claims: &Claims,
    ) -> Result<u64, AppError> {
        let user_record = sqlx::query!(
            r#"SELECT id FROM users WHERE username = ?"#,
            claims.username
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_err!("Failed to get user"))?
        .ok_or_else(|| AppError::NotFound(format!("User not found: {}", claims.username)))?;

        // Before create a new order inspection record, check if order inspections table has a record with the same order_id
        // get the inspection_round from the order inspections table
        let last_inspection = sqlx::query!(
            r#"SELECT id, inspection_round, inspected_by_type FROM order_inspections WHERE order_id = ? ORDER BY inspection_round DESC LIMIT 1"#,
            order_id
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_err!("Failed to get inspection round"))?;

        let last_inspection_id = last_inspection.as_ref().map(|r| r.id as u64);

        let (parent_id, inspection_round) = if let Some(inspection_record) = last_inspection {
            (
                Some(inspection_record.id),
                inspection_record.inspection_round + 1,
            )
        } else {
            (None, 1)
        };

        // Create a new order inspection record
        let current_inspection_id = sqlx::query!(
            r#"INSERT INTO order_inspections (order_id, inspected_by_type, inspected_by_id, inspection_round, parent_id) VALUES (?, ?, ?, ?, ?)"#,
            order_id, claims.tenant_type, user_record.id, inspection_round, parent_id
        )
        .execute(&mut **tx)
        .await
        .map_err(map_db_err!("Failed to create order inspection record"))?
        .last_insert_id();

        if current_inspection_id == 0 {
            return Err(AppError::Internal(format!(
                "Failed to create order inspection record"
            )));
        }

        match TenantType::try_from(claims.tenant_type.as_str())? {
            TenantType::Market => {
                self.insert_order_inspection_items_from_provider_delivery(
                    tx,
                    order_id,
                    current_inspection_id,
                )
                .await?
            }
            TenantType::Customer => {
                self.insert_order_inspection_items_from_last_market_inspection(
                    tx,
                    order_id,
                    last_inspection_id.unwrap(),
                    current_inspection_id,
                )
                .await?
            }
            TenantType::Provider => {
                return Err(AppError::BadRequest(format!(
                    "Provider tenant type not supported for this operation"
                )))
            }
        }

        Ok(current_inspection_id)
    }

    async fn insert_order_inspection_items_from_provider_delivery(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
        order_id: i32,
        current_inspection_id: u64,
    ) -> Result<(), AppError> {
        let current_delivery = sqlx::query!(
            r#"SELECT id as delivery_id, delivery_type FROM provider_deliveries 
               WHERE assignment_id = ? ORDER BY delivery_round DESC LIMIT 1"#,
            order_id
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_err!("Failed to get max round"))?
        .ok_or_else(|| AppError::NotFound(format!("Max round not found: {}", order_id)))?;

        let provider_delivery_items = sqlx::query!(
            r#"INSERT INTO order_inspection_items (inspection_id, order_detail_id, need_to_inspection) 
               SELECT ? AS inspection_id, pdi.order_detail_id, pdi.actual_qty FROM provider_delivery_items pdi WHERE pdi.delivery_id = ?"#, 
               current_inspection_id, current_delivery.delivery_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_err!("Failed to insert order inspection items"))?
            .rows_affected();

        if provider_delivery_items == 0 {
            return Err(AppError::Internal(format!(
                "initialize order inspection items failed: {}",
                current_delivery.delivery_id
            )));
        }

        Ok(())
    }

    async fn insert_order_inspection_items_from_last_market_inspection(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
        order_id: i32,
        last_inspection_id: u64,
        current_inspection_id: u64,
    ) -> Result<(), AppError> {
        // 获取CUSTOMER最近一次的验收记录
        let customer_last_inspection = sqlx::query!(
            r#"
               SELECT count(*) as count 
               FROM order_inspections 
               WHERE order_id = ? AND inspected_by_type = 'CUSTOMER'"#,
            order_id
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_err!("Failed to get last inspection"))?;

        // 如果CUSTOMER只有一个验收记录，则说明是刚创建的，即CUSTOMER还没有验收过
        if customer_last_inspection.count == 1 {
            let last_inspection_items = sqlx::query!(r#"
               select order_detail_id, inspected_qty from order_inspection_items oii 
               inner join order_inspections oi on oi.id = oii.inspection_id 
               inner join orders o on o.id = oi.order_id 
                where o.id = ? and oii.result != 'RETURN' AND oi.inspected_by_type = 'MARKET'
            "#, 
            order_id)
           .fetch_all(&mut **tx)
           .await
           .map_err(map_db_err!("Failed to get last inspection items"))?;

            // 根据 order_detail_id 聚合 need_to_inspection
            let mut need_to_inspect_map = HashMap::new();
            for item in last_inspection_items {
                let entry = need_to_inspect_map
                    .entry(item.order_detail_id)
                    .or_insert(Decimal::ZERO);
                *entry = *entry + item.inspected_qty.unwrap_or(Decimal::ZERO);
            }

            if !need_to_inspect_map.is_empty() {
                // 使用批量插入优化性能，只插入 need_to_inspection，不插入 inspected_qty（因为还没有开始验收）
                let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO order_inspection_items (inspection_id, order_detail_id, need_to_inspection)"
            );

                query_builder.push_values(
                    need_to_inspect_map,
                    |mut b, (order_detail_id, need_to_inspection)| {
                        b.push_bind(current_inspection_id)
                            .push_bind(order_detail_id)
                            .push_bind(need_to_inspection);
                    },
                );

                let query = query_builder.build();
                query
                    .execute(&mut **tx)
                    .await
                    .map_err(map_db_err!("Failed to batch insert order inspection items"))?;
            }
        } else {
            // 直接从上一轮的验收记录SELECT并插入
            sqlx::query!(
                r#"
                INSERT INTO order_inspection_items (
                    inspection_id,
                    order_detail_id,
                    need_to_inspection
                )
                SELECT 
                    ?, order_detail_id, inspected_qty
                FROM order_inspection_items
                WHERE inspection_id = ? AND accepted = 1
                "#,
                current_inspection_id,
                last_inspection_id,
            )
            .execute(&mut **tx)
            .await
            .map_err(map_db_err!("Failed to insert order inspection items"))?;
        }

        Ok(())
    }


    /// Send messages to all users of tenants associated with an order
    ///
    /// # Arguments
    /// * `order_code` - The order code to find associated tenants
    /// * `category` - Message category
    /// * `title` - Message title
    /// * `content` - Message content
    ///
    /// # Returns
    /// Number of messages sent
    pub(crate) async fn send_order_messages_to_tenants(
        &self,
        order_id: i32,
        category: &str,
        title: &str,
        content: &str,
        exclude_tenant_type: String,
    ) -> Result<u64, AppError> {
        // Get order information including customer_id, market_id, and provider_id in one query
        let order_info = sqlx::query!(
            r#"
            SELECT o.customer_id, o.market_id, o.order_status, poa.provider_id
            FROM orders o
            LEFT JOIN provider_orders_assignments poa ON o.id = poa.order_id
            WHERE o.id = ? AND o.deleted_at IS NULL
            "#,
            order_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order information"))?
            .ok_or_else(|| AppError::not_found(format!("Order not found: {}", order_id)))?;

        let provider_id = order_info.provider_id;

        // 解析 exclude_tenant_type，确定需要排除的租户类型
        let exclude_type = exclude_tenant_type.to_ascii_uppercase();
        let exclude_customer = exclude_type == "CUSTOMER";
        let exclude_market = exclude_type == "MARKET";
        let exclude_provider = exclude_type == "PROVIDER";

        // Build query to get all user IDs for these tenants
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT u.id
            FROM users u
            WHERE u.tenant_id IN (
            "#,
        );

        let mut tenant_ids_added = false;

        // 添加 customer_id（如果不需要排除）
        if !exclude_customer {
            query_builder.push_bind(order_info.customer_id);
            tenant_ids_added = true;
        }

        // 添加 market_id（如果不需要排除）
        if !exclude_market {
            if tenant_ids_added {
                query_builder.push(",");
            }
            query_builder.push_bind(order_info.market_id);
            tenant_ids_added = true;
        }

        // 添加 provider_id（如果存在且不需要排除）
        if !exclude_provider {
            if let Some(pid) = provider_id {
                if tenant_ids_added {
                    query_builder.push(",");
                }
                query_builder.push_bind(pid);
                tenant_ids_added = true;
            }
        }

        // 如果没有有效的租户ID，直接返回0
        if !tenant_ids_added {
            return Ok(0);
        }

        query_builder.push(") AND u.deleted_at IS NULL");

        let user_ids: Vec<i32> = query_builder
            .build_query_scalar::<i32>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get user IDs"))?;

        if user_ids.is_empty() {
            return Ok(0);
        }

        // Batch insert messages
        let mut message_builder = sqlx::QueryBuilder::new(
            "INSERT INTO messages (user_id, category, title, content) ",
        );

        message_builder.push_values(user_ids.iter(), |mut b, user_id| {
            b.push_bind(user_id)
                .push_bind(category)
                .push_bind(title)
                .push_bind(content);
        });

        let result = message_builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Failed to batch insert messages"))?;

        Ok(result.rows_affected())
    }
}
