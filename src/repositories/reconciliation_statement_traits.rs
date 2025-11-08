use crate::common::AppError;
use crate::dto::reconciliation_statement::ReconciliationStatementOrderDTO;
use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{dto::reconciliation_statement::ReconciliationStatementDTO, map_db_err};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use tracing::{debug, error, info, warn};

use super::{generate_code, CodeType};

#[async_trait]
pub trait ReconciliationStatementRepository: Send + Sync {
    async fn create_statements(&self) -> Result<Vec<i32>, AppError>;

    async fn create_statement_for_combination(
        &self,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
        customer_id: i32,
        market_id: i32,
        provider_id: i32,
        statement_code: &str,
    ) -> Result<i32, AppError>;

    async fn get_statements_by_tenant_and_date(
        &self,
        name_hash: &str,
        tenant_type: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<ReconciliationStatementDTO>, AppError>;

    async fn get_statement_orders_by_id(
        &self,
        name_hash: &str,
        tenant_type: &str,
        id: i32,
    ) -> Result<Vec<ReconciliationStatementOrderDTO>, AppError>;
}

#[async_trait]
impl ReconciliationStatementRepository for MySqlRepository {
    async fn create_statements(&self) -> Result<Vec<i32>, AppError> {
        // 1. 计算当月的开始和结束日期
        let now = chrono::Local::now();
        let current_month_start = NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
            .ok_or_else(|| AppError::Internal("无法创建当月开始日期".to_string()))?;

        debug!("current_month_start: {}", current_month_start);

        // 计算当月结束日期
        let next_month_start = if now.month() == 12 {
            // 如果是12月，下个月是下一年的1月
            NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
        } else {
            // 否则下个月是当年的下一个月
            NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
        }
        .ok_or_else(|| AppError::Internal("无法创建下月开始日期".to_string()))?;

        // 当月结束日期是下个月第一天减去1天
        let current_month_end = next_month_start
            .pred_opt()
            .ok_or_else(|| AppError::Internal("无法计算当月结束日期".to_string()))?;

        // 2. 获取需要对账的客户-市场-供应商组合
        let combinations = sqlx::query!(
            r#"
            SELECT DISTINCT o.customer_id, o.market_id, po.provider_id
            FROM orders o
            JOIN provider_orders_assignments po ON o.id = po.order_id
            WHERE o.created_at BETWEEN ? AND ?
            AND o.order_status = 'COMPLETED'
            AND o.deleted_at IS NULL
            "#,
            current_month_start,
            current_month_end
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to fetch combinations"))?;

        let mut statement_ids = Vec::new();
        let mut failed_combinations = Vec::new();

        // 3. 为每个组合创建对账单（使用单独的事务）
        for combo in combinations {
            let customer_id = combo.customer_id;
            let market_id = combo.market_id;
            let provider_id = combo.provider_id;

            // 3.1 检查该组合的对账单是否已存在
            let exists = sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(1) FROM reconciliation_statements
                WHERE start_date = ? 
                AND end_date = ?
                AND customer_id = ?
                AND market_id = ?
                AND provider_id = ?
                AND deleted_at IS NULL
                "#,
            )
            .bind(current_month_start)
            .bind(current_month_end)
            .bind(customer_id)
            .bind(market_id)
            .bind(provider_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!(
                "Failed to check if reconciliation statement exists"
            ))?;

            if exists > 0 {
                // 该组合已有对账单，跳过
                debug!(
                    "对账单已存在: customer_id={}, market_id={}, provider_id={}",
                    customer_id, market_id, provider_id
                );
                continue;
            }

            // 3.2 为该组合创建对账单（单独事务）
            let statement_code = generate_code(CodeType::ReconciliationStatement);

            match self
                .create_statement_for_combination(
                    &current_month_start,
                    &current_month_end,
                    customer_id,
                    market_id,
                    provider_id,
                    &statement_code,
                )
                .await
            {
                Ok(id) => {
                    statement_ids.push(id);
                    info!(
                        "成功创建对账单: id={}, customer_id={}, market_id={}, provider_id={}",
                        id, customer_id, market_id, provider_id
                    );
                }
                Err(e) => {
                    error!(
                        "创建对账单失败: customer_id={}, market_id={}, provider_id={}, error={}",
                        customer_id, market_id, provider_id, e
                    );
                    failed_combinations.push((customer_id, market_id, provider_id));
                    // 继续处理其他组合，不中断整体流程
                }
            }
        }

        // 4. 汇总结果
        if !failed_combinations.is_empty() {
            // 记录失败的组合，但不影响成功的部分
            warn!("部分对账单创建失败: {:?}", failed_combinations);
        }

        Ok(statement_ids)
    }

    // 为单个组合创建对账单（使用独立事务）
    async fn create_statement_for_combination(
        &self,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
        customer_id: i32,
        market_id: i32,
        provider_id: i32,
        statement_code: &str,
    ) -> Result<i32, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // 1. 创建对账单主表记录（暂时金额为0，后面会更新）
        let statement_id = sqlx::query!(
            r#"
            INSERT INTO reconciliation_statements (
                statement_code, customer_id, market_id, provider_id,
                start_date, end_date, total_amount, discount_amount,
                actual_amount, status, created_by, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, 0, 0, 0, 'PENDING', 'system', NOW(), NOW())
            "#,
            statement_code,
            customer_id,
            market_id,
            provider_id,
            start_date,
            end_date
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to create reconciliation statement"))?
        .last_insert_id() as i32;

        // 2. 查询该组合下的已完成订单
        let orders = sqlx::query!(
            r#"
            SELECT o.id, o.order_code, DATE(o.created_at) as order_date,
                    o.total_amount, o.discount_amount, o.actual_amount
            FROM orders o
            JOIN provider_orders_assignments po ON o.id = po.order_id
            WHERE o.customer_id = ?
            AND o.market_id = ?
            AND po.provider_id = ?
            AND o.created_at BETWEEN ? AND ?
            AND o.order_status = 'COMPLETED'
            AND o.deleted_at IS NULL
            "#,
            customer_id,
            market_id,
            provider_id,
            start_date,
            end_date
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to fetch orders"))?;

        // 如果没有订单，则回滚事务并返回错误
        if orders.is_empty() {
            tx.rollback()
                .await
                .map_err(map_db_err!("Failed to rollback transaction"))?;
            return Err(AppError::NotFound(format!(
                "未找到需要对账的订单: customer_id={}, market_id={}, provider_id={}",
                customer_id, market_id, provider_id
            )));
        }

        // 3. 插入对账单订单关联表和计算金额总和
        let mut total_amount_sum = 0.0;
        let mut discount_amount_sum = 0.0;
        let mut actual_amount_sum = 0.0;

        for order in &orders {
            // 3.1 插入对账单订单关联表
            sqlx::query!(
                r#"
                INSERT INTO reconciliation_statement_orders (
                    statement_id, order_id, order_code, order_date,
                    total_amount, actual_amount, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, NOW())
                "#,
                statement_id,
                order.id,
                &order.order_code,
                &order.order_date,
                &order.total_amount,
                &order.actual_amount
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!(
                "Failed to create reconciliation statement order relation"
            ))?;

            // 3.2 查询订单详情
            let order_details = sqlx::query!(
                r#"
                SELECT od.id, od.product_code, od.product_name, od.category_name, 
                      od.unit, od.quantity, od.accepted_quantity,
                      od.original_price, od.actual_price,
                      od.total_amount, od.actual_amount
                FROM order_details od
                WHERE od.order_id = ?
                "#,
                order.id
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to fetch order details"))?;

            for detail in &order_details {
                // 按照reconciliation_statement_details表的实际结构插入数据
                sqlx::query!(
                    r#"
                    INSERT INTO reconciliation_statement_details (
                        statement_id, order_detail_id, product_code, product_name,
                        category_name, unit, original_quantity, actual_quantity,
                        receipt_quantity, returned_quantity, price, original_amount, actual_amount,
                        order_date, created_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?, ?, NOW())
                    "#,
                    statement_id,
                    detail.id,
                    &detail.product_code,
                    &detail.product_name,
                    &detail.category_name,
                    &detail.unit,
                    &detail.quantity,
                    &detail.accepted_quantity,
                    &detail.accepted_quantity, // 假设收货数量等于实际数量
                    &detail.actual_price,
                    &detail.total_amount,
                    &detail.actual_amount,
                    &order.order_date
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!(
                    "Failed to create reconciliation statement detail"
                ))?;
            }

            // 3.3 累加金额
            total_amount_sum += order.total_amount.to_string().parse::<f64>().unwrap_or(0.0);
            discount_amount_sum += order
                .discount_amount
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0);
            actual_amount_sum += order
                .actual_amount
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0);
        }

        // 4. 更新对账单主表的金额
        sqlx::query!(
            r#"
            UPDATE reconciliation_statements
            SET total_amount = ?, discount_amount = ?, actual_amount = ?, updated_at = NOW()
            WHERE id = ?
            "#,
            total_amount_sum,
            discount_amount_sum,
            actual_amount_sum,
            statement_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_db_err!(
            "Failed to update reconciliation statement amounts"
        ))?;

        // 5. 提交事务
        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;

        Ok(statement_id)
    }

    async fn get_statements_by_tenant_and_date(
        &self,
        name_hash: &str,
        tenant_type: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<ReconciliationStatementDTO>, AppError> {
        match tenant_type {
            "CUSTOMER" => {
                let statements = sqlx::query_as::<_, ReconciliationStatementDTO>(
                    r#"
                    SELECT 
                        rs.id, rs.statement_code, rs.customer_id, rs.market_id, rs.provider_id,
                        pt.name as supplier_name, t.name as customer_name,
                        rs.start_date, rs.end_date, rs.total_amount, rs.discount_amount, rs.actual_amount,
                        rs.status, rs.remark, rs.created_by, rs.confirmed_by, rs.confirmed_at,
                        rs.completed_by, rs.completed_at, rs.created_at, rs.updated_at, rs.deleted_at
                    FROM reconciliation_statements rs 
                    JOIN tenants t ON t.id = rs.customer_id
                    JOIN tenants pt ON pt.id = rs.provider_id AND pt.tenant_type = 'PROVIDER'
                    WHERE t.name_hash = ? AND t.tenant_type = ? AND rs.start_date BETWEEN ? AND ?
                    ORDER BY rs.created_at DESC"#
                )
                .bind(name_hash)
                .bind(tenant_type)
                .bind(start_date)
                .bind(end_date)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to fetch reconciliation statements"))?;

                Ok(statements)
            }
            "PROVIDER" => {
                let statements = sqlx::query_as::<_, ReconciliationStatementDTO>(
                    r#"
                    SELECT 
                        rs.id, rs.statement_code, rs.customer_id, rs.market_id, rs.provider_id,
                        t.name as supplier_name, mt.name as customer_name,
                        rs.start_date, rs.end_date, rs.total_amount, rs.discount_amount, rs.actual_amount,
                        rs.status, rs.remark, rs.created_by, rs.confirmed_by, rs.confirmed_at,
                        rs.completed_by, rs.completed_at, rs.created_at, rs.updated_at, rs.deleted_at
                    FROM reconciliation_statements rs 
                    JOIN tenants t ON t.id = rs.provider_id
                    JOIN tenants mt ON mt.id = rs.customer_id AND mt.tenant_type = 'CUSTOMER'
                    WHERE t.name_hash = ? AND t.tenant_type = ? AND rs.start_date BETWEEN ? AND ?
                    ORDER BY rs.created_at DESC"#
                )
                .bind(name_hash)
                .bind(tenant_type)
                .bind(start_date)
                .bind(end_date)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to fetch reconciliation statements"))?;

                Ok(statements)
            }
            "MARKET" => {
                let statements = sqlx::query_as::<_, ReconciliationStatementDTO>(
                    r#"
                    SELECT 
                        rs.id, rs.statement_code, rs.customer_id, rs.market_id, rs.provider_id,
                        pt.name as supplier_name, mt.name as customer_name,
                        rs.start_date, rs.end_date, rs.total_amount, rs.discount_amount, rs.actual_amount,
                        rs.status, rs.remark, rs.created_by, rs.confirmed_by, rs.confirmed_at,
                        rs.completed_by, rs.completed_at, rs.created_at, rs.updated_at, rs.deleted_at
                    FROM reconciliation_statements rs 
                    JOIN tenants t ON t.id = rs.market_id
                    JOIN tenants pt ON pt.id = rs.provider_id AND pt.tenant_type = 'PROVIDER'
                    JOIN tenants mt ON mt.id = rs.customer_id AND mt.tenant_type = 'CUSTOMER'
                    WHERE t.name_hash = ? AND t.tenant_type = ? AND rs.start_date BETWEEN ? AND ?
                    ORDER BY rs.created_at DESC"#
                )
                .bind(name_hash)
                .bind(tenant_type)
                .bind(start_date)
                .bind(end_date)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to fetch reconciliation statements"))?;

                Ok(statements)
            }
            _ => Err(AppError::Internal("Invalid tenant type".to_string())),
        }
    }

    async fn get_statement_orders_by_id(
        &self,
        name_hash: &str,
        tenant_type: &str,
        id: i32,
    ) -> Result<Vec<ReconciliationStatementOrderDTO>, AppError> {
        match tenant_type {
            "CUSTOMER" => {
                let orders = sqlx::query_as::<_, ReconciliationStatementOrderDTO>(
                    r#"
                    SELECT 
                        ro.id, ro.statement_id, ro.order_id, ro.order_code, ro.order_date, ro.total_amount, ro.actual_amount,
                        ro.created_at
                    FROM reconciliation_statement_orders ro 
                    JOIN reconciliation_statements rs ON rs.id = ro.statement_id 
                    JOIN tenants t ON t.id = rs.customer_id 
                    WHERE t.name_hash = ? AND t.tenant_type = ? AND ro.statement_id = ?
                    "#
                )
                .bind(name_hash)
                .bind(tenant_type)
                .bind(id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to fetch reconciliation statement orders"))?;

                Ok(orders)
            }
            "PROVIDER" => {
                let orders = sqlx::query_as::<_, ReconciliationStatementOrderDTO>(
                    r#"
                    SELECT 
                        ro.id, ro.statement_id, ro.order_id, ro.order_code, ro.order_date, ro.total_amount, ro.actual_amount,
                        ro.created_at
                    FROM reconciliation_statement_orders ro     
                    JOIN reconciliation_statements rs ON rs.id = ro.statement_id 
                    JOIN tenants t ON t.id = rs.provider_id 
                    WHERE t.name_hash = ? AND t.tenant_type = ? AND ro.statement_id = ?
                    "#
                )
                .bind(name_hash)
                .bind(tenant_type)
                .bind(id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to fetch reconciliation statement orders"))?;

                Ok(orders)
            }
            "MARKET" => {
                let orders = sqlx::query_as::<_, ReconciliationStatementOrderDTO>(
                    r#"
                    SELECT 
                        ro.id, ro.statement_id, ro.order_id, ro.order_code, ro.order_date, ro.total_amount, ro.actual_amount,
                        ro.created_at
                    FROM reconciliation_statement_orders ro 
                    JOIN reconciliation_statements rs ON rs.id = ro.statement_id 
                    JOIN tenants t ON t.id = rs.market_id 
                    WHERE t.name_hash = ? AND t.tenant_type = ? AND ro.statement_id = ?
                    "#
                )
                .bind(name_hash)
                .bind(tenant_type)
                .bind(id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to fetch reconciliation statement orders"))?;

                Ok(orders)
            }
            _ => Err(AppError::Internal("Invalid tenant type".to_string())),
        }
    }
}
