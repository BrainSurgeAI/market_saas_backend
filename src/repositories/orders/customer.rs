use crate::{
    common::AppError,
    dto::order::CreateOrderDTO,
    // 以下导入已注释，如果将来需要查询分类层级映射时，可以取消注释
    // dto::category::CategoryLevel1Row,
    map_db_err,
    repositories::{my_sql_repository::MySqlRepository, generate_code, CodeType},
    models::{claims::Claims, order_action::OrderAction},
};

use async_trait::async_trait;

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use tracing::{debug, error, info};
use sqlx::{MySql, QueryBuilder};

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
    async fn create_order(&self, claims: &Claims, order_payload: &CreateOrderDTO) -> Result<String, AppError>;
}

impl MySqlRepository {
    /// 验证订单金额是否正确
    /// 
    /// 根据每个商品的 product_code, category_id 和 customer_id 查找对应的折扣，
    /// 计算每个商品的总金额，累加得到订单总金额和折扣金额，
    /// 与传入的 total_amount 进行对比验证
    async fn verify_order_amount(&self, customer_id: i32, order_payload: &CreateOrderDTO) -> Result<(), AppError> {

        debug!("开始验证订单金额: 客户ID={}, 订单={:?}", customer_id, order_payload);
        // 解析配送日期，用于查询折扣的有效期
        let delivery_date = NaiveDate::parse_from_str(
            &order_payload.delivery_info.delivery_date,
            "%Y-%m-%d",
        )
        .map_err(|e| {
            error!("配送日期格式错误: {:#?}", e);
            AppError::bad_request(format!(
                "配送日期格式错误: {}",
                order_payload.delivery_info.delivery_date
            ))
        })?;

        // 检查配送日期不能早于今天
        let today = Utc::now().date_naive();
        if delivery_date < today {
            error!("配送日期不能早于今天: 配送日期={}, 今天={}", delivery_date, today);
            return Err(AppError::bad_request(format!(
                "配送日期不能早于今天，配送日期: {}",
                delivery_date
            )));
        }

        debug!("配送日期验证成功: 配送日期={}, 今天={}", delivery_date, today);

        // 收集所有需要查询的 category_id
        let category_ids: Vec<i32> = order_payload
            .items
            .iter()
            .map(|item| item.category_id)
            .collect();

        if category_ids.is_empty() {
            error!("订单商品列表不能为空: 订单={:?}", order_payload);
            return Err(AppError::bad_request("订单商品列表不能为空"));
        }

        // 当前实现：订单中的 category_id 已经是一级分类，直接使用
        // 构建 category_id -> level1_category_id 的映射（直接映射，因为 category_id 就是一级分类）
        let category_to_level1: std::collections::HashMap<i32, i32> = category_ids
            .iter()
            .map(|&category_id| (category_id, category_id))
            .collect();

        debug!("构建的分类映射数量: {}, 订单中的分类ID: {:?}", category_to_level1.len(), category_ids);

        // ========== 以下代码已注释，如果将来需求变化，订单中的 category_id 可能是二级或三级分类时，可以取消注释使用 ==========
        // 
        // // 查询每个分类对应的一级分类ID
        // // 因为折扣表存储的是一级分类的折扣，需要根据分类级别向上查找一级分类
        // // 支持一级、二级、三级分类
        // let mut query_builder = QueryBuilder::<MySql>::new(
        //     r#"
        //     SELECT DISTINCT
        //         c.id as category_id,
        //         CASE 
        //             WHEN c.level = 1 THEN c.id
        //             WHEN c.level = 2 THEN c.parent_id
        //             WHEN c.level = 3 THEN COALESCE(c1.id, c.parent_id)
        //             ELSE NULL
        //         END as level1_category_id
        //     FROM categories c
        //     LEFT JOIN categories c2 ON c.level = 3 AND c.parent_id = c2.id AND c2.level = 2
        //     LEFT JOIN categories c1 ON c.level = 3 AND c2.parent_id = c1.id AND c1.level = 1
        //     WHERE c.id IN (
        //     "#
        // );
        // 
        // // 手动构建 IN 子句
        // for (index, category_id) in category_ids.iter().enumerate() {
        //     if index > 0 {
        //         query_builder.push(", ");
        //     }
        //     query_builder.push_bind(category_id);
        // }
        // 
        // query_builder.push(")");
        //
        // let category_level1_map: Vec<CategoryLevel1Row> = query_builder
        //     .build_query_as()
        //     .fetch_all(&self.pool)
        //     .await
        //     .map_err(map_db_err!("Failed to query category level1 mapping"))?;
        //
        // debug!("查询分类一级分类映射成功: 分类一级分类映射数量={}", category_level1_map.len());
        //
        // // 构建 category_id -> level1_category_id 的映射
        // let mut category_to_level1: std::collections::HashMap<i32, i32> = 
        //     std::collections::HashMap::new();
        // for row in category_level1_map {
        //     if let Some(level1_id) = row.level1_category_id {
        //         category_to_level1.insert(row.category_id, level1_id);
        //     } else {
        //         error!("分类 {} 无法找到一级分类", row.category_id);
        //     }
        // }
        //
        // debug!("构建的分类映射数量: {}, 订单中的分类ID: {:?}", category_to_level1.len(), category_ids);
        // ========== 注释结束 ==========

        // 查询所有相关的折扣信息
        let level1_category_ids: Vec<i32> = category_to_level1
            .values()
            .copied()
            .collect();

        if level1_category_ids.is_empty() {
            error!("无法找到商品分类的一级分类: 订单={:?}", order_payload);
            return Err(AppError::bad_request("无法找到商品分类的一级分类"));
        }

        debug!("开始查询折扣信息: 客户ID={}, 订单={:?}", customer_id, order_payload);

        // 查询折扣信息
        let mut discount_query_builder = QueryBuilder::<MySql>::new(
            r#"
            SELECT category_id, discount_rate
            FROM customer_category_discounts
            WHERE tenant_id = 
            "#
        );
        discount_query_builder.push_bind(customer_id);
        discount_query_builder.push(" AND category_id IN (");
        
        // 手动构建 IN 子句
        for (index, category_id) in level1_category_ids.iter().enumerate() {
            if index > 0 {
                discount_query_builder.push(", ");
            }
            discount_query_builder.push_bind(category_id);
        }
        
        discount_query_builder.push(
            r#"
            ) AND status = true
              AND start_date <= 
            "#
        );
        discount_query_builder.push_bind(delivery_date);
        discount_query_builder.push(
            r#"
              AND (end_date IS NULL OR end_date >= 
            "#
        );
        discount_query_builder.push_bind(delivery_date);
        discount_query_builder.push(") ORDER BY start_date DESC");

        #[derive(sqlx::FromRow)]
        struct DiscountRow {
            category_id: i32,
            discount_rate: Decimal,
        }

        let discounts: Vec<DiscountRow> = discount_query_builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to query customer discounts"))?;

        // 构建 level1_category_id -> discount_rate 的映射
        // 如果有多个折扣记录，取最新的（start_date DESC 排序后的第一条）
        let mut discount_map: std::collections::HashMap<i32, Decimal> = 
            std::collections::HashMap::new();
        for discount in discounts {
            // 如果该分类已经有折扣记录，跳过（因为已经按 start_date DESC 排序，第一条是最新的）
            discount_map.entry(discount.category_id).or_insert(discount.discount_rate);
        }

        // 计算每个商品的总金额和折扣金额
        let mut calculated_total_amount = Decimal::ZERO;
        let mut calculated_discount_amount = Decimal::ZERO;

        for item in &order_payload.items {
            // 获取一级分类ID
            let level1_category_id = category_to_level1
                .get(&item.category_id)
                .ok_or_else(|| {
                    AppError::bad_request(format!(
                        "无法找到商品 {} 的一级分类",
                        item.product_code
                    ))
                })?;

            // 获取折扣率，如果没有折扣则默认为 1.0
            let discount_rate = discount_map
                .get(level1_category_id)
                .copied()
                .unwrap_or(Decimal::ONE);

            // 计算商品总金额：数量 * 原始单价 * 折扣率
            let item_total = item.quantity * item.original_price * discount_rate;
            
            // 计算折扣金额：原始金额 - 折扣后金额
            let item_discount = item.original_amount - item_total;

            calculated_total_amount += item_total;
            calculated_discount_amount += item_discount;
        }

        // 计算实际支付金额 = 总金额 - 折扣金额
        let calculated_actual_amount = calculated_total_amount;

        // 允许的误差范围：0.01 元（由于浮点数精度问题）
        let tolerance = Decimal::new(1, 2); // 0.01

        // 验证总金额（这里验证的是 actual_amount，因为订单的 total_amount 是原始总金额）
        let amount_diff = (calculated_actual_amount - order_payload.total_amount).abs();
        if amount_diff > tolerance {
            error!(
                "订单金额验证失败: 计算金额={}, 传入金额={}, 差异={}",
                calculated_actual_amount, order_payload.total_amount, amount_diff
            );
            return Err(AppError::bad_request(format!(
                "订单金额验证失败: 计算金额 {}, 传入金额 {}, 差异超过允许范围(0.01): {} 元",
                calculated_actual_amount, order_payload.total_amount, amount_diff
            )));
        }

        info!("订单金额验证成功: 计算金额={}, 传入金额={}, 差异={}", calculated_actual_amount, order_payload.total_amount, amount_diff);

        Ok(())
    }
}

#[async_trait]
impl CustomerOrderRepository for MySqlRepository {
    async fn create_order(&self, claims: &Claims, order_payload: &CreateOrderDTO) -> Result<String, AppError> {
         // Fetch customer information with early return for better error handling
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

        // 验证订单金额
        self.verify_order_amount(record.customer_id, order_payload).await?;

        // Pre-calculate order amounts for efficiency
        let total_amount: Decimal = order_payload.items.iter().map(|item| item.original_amount).sum();

        let discount_amount: Decimal = order_payload
            .items
            .iter()
            .map(|item| item.original_amount - item.total)
            .sum();
     
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
            order_payload.total_amount,
            order_payload.delivery_info.delivery_date,
            order_payload.delivery_info.delivery_address,
            order_payload.delivery_info.contact_name,
            order_payload.delivery_info.contact_phone,
            claims.real_name)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to create order"))?
        .last_insert_id();

        // Batch insert order details for better performance
        self.insert_order_details(&mut tx, order_id, &order_payload.items)
            .await
            .map_err(|e| {
                error!("Failed to create order details: {:#?}", e);
                AppError::Internal(e.to_string())
            })?;

        // create a record in order_status_history table
        sqlx::query!(
            r#"INSERT INTO order_status_history (order_id, from_status, to_status, changed_by, change_reason) VALUES (?, ?, ?, ?, ?)"#,
            order_id,
            "CREATED".to_string(),
            "PENDING".to_string(),
            claims.real_name,
            OrderAction::Create.description()
            
        )
        .execute(&mut *tx)
        .await.map_err(map_db_err!("Failed to create order status history"))?;
        // Commit transaction
        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        info!("Order {} created successfully", order_code);

        Ok(order_code)
    }
}