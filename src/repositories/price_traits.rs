use super::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::price::{
        PriceAnnouncement, PriceApprovalParam, PriceCreateDTO, PriceQueryParams,
        PriceStatusPaginationParams,
    },
    dto::products::{PaginatedProductDailyPriceComparison, ProductDailyPriceComparisonDTO},
    map_db_err,
};
use async_trait::async_trait;
use chrono::{Local, NaiveTime};
use sqlx::{MySql, QueryBuilder};
use tracing::debug;

#[async_trait]
pub(crate) trait PriceRepository: Send + Sync {
    /// Fetches published price announcements with optional filtering by category, product name, and date.
    ///
    /// This function dynamically builds and executes a SQL query to retrieve the latest published prices
    /// for products, including their price changes (min, avg, max) compared to the previous record.
    ///
    /// ### Date logic
    /// - If `params.date` is **provided**, fetch price announcements for that date.
    /// - If `params.date` is **not provided**:
    ///   - Before **12:00 PM**, the query fetches announcements for **the previous day**.
    ///   - After **12:00 PM**, the query fetches announcements for **the current day**.
    ///
    /// ### Category and product filtering
    /// - If `params.category_l1` (Level 1) is **provided**, the query filters by that category.
    ///   - Otherwise, defaults to category with ID `759`.
    /// - If `params.category_l3` (Level 3) is **provided**, it filters by that subcategory.
    /// - If `params.name` is **provided**, it performs a fuzzy search on the product name.
    /// - If none of the above filters are provided, all active (non-disabled) products under category `759` are returned.
    ///
    /// ### Returned columns
    /// Each [`PriceAnnouncement`] record includes:
    /// - `level1_category`: Name of the level 1 category  
    /// - `level3_category`: Name of the level 3 category  
    /// - `product_name`, `product_code`, `unit`
    /// - `min_price`, `avg_price`, `max_price`
    /// - `min_price_change`, `avg_price_change`, `max_price_change`
    /// - `price_status`: Either `"PUBLISHED"` or `"NOT_PUBLISHED"`
    /// - `price_date`: The date associated with the published price
    ///
    /// ### Implementation details
    /// - Uses [`sqlx::QueryBuilder`] to dynamically assemble a complex SQL query.  
    /// - Joins category hierarchy (`level1 → level2 → level3`) and product tables.  
    /// - Uses a subquery to identify the latest published price date per product,
    ///   handling cases where the current date has no published record.
    /// - Computes price changes based on whether the current time is before or after noon.
    ///
    /// # Parameters
    ///
    /// * `params` — [`PriceQueryParams`], which may include:
    ///   - `date`: Specific date to query
    ///   - `category_l1`: Level 1 category ID
    ///   - `category_l3`: Level 3 category ID
    ///   - `name`: Product name (supports partial match)
    ///
    /// # Returns
    ///
    /// A vector of [`PriceAnnouncement`] objects representing published product prices and their variations.
    ///
    /// # Errors
    ///
    /// Returns [`AppError`] if the database query or data mapping fails.

    async fn list_price_announcements_by_category_product_name_and_date(
        &self,
        query: &PriceQueryParams,
    ) -> Result<Vec<PriceAnnouncement>, AppError>;

    async fn list_pricer_daily_price_comparison(
        &self,
        username: &str,
        role_name: &str,
        query: &PriceStatusPaginationParams,
    ) -> Result<PaginatedProductDailyPriceComparison, AppError>;

    /// 获取用户关联分类中指定价格状态的产品列表
    ///
    /// 查找与用户关联的产品分类中，当天价格状态为指定状态的产品
    /// 支持PENDING（待审核）、REJECTED（已拒绝）、PUBLISHED（已发布）状态
    /// 返回分页数据，包含历史价格信息作为参考
    ///
    /// # Arguments
    /// * `username` - 用户名
    /// * `role_name` - 用户角色名
    /// * `query` - 分页查询参数，包含status字段用于指定价格状态
    ///
    /// # Returns
    /// * `PaginatedProductDailyPriceComparison` - 包含分页信息和指定状态产品数据
    ///
    /// # Errors
    /// Returns an `AppError` if:
    /// - Database query fails
    /// - User has no category assignments
    /// - Invalid role name is provided
    /// - Invalid status value is provided
    async fn list_price_products_by_status(
        &self,
        username: &str,
        role_name: &str,
        query: &PriceStatusPaginationParams,
    ) -> Result<PaginatedProductDailyPriceComparison, AppError>;

    /// 通用辅助函数：执行用户产品分页查询
    async fn query_prices_paginated(
        &self,
        username: &str,
        role_name: &str,
        query: &PriceStatusPaginationParams,
        where_clause: &str,
        select_fields: &str,
        additional_joins: &str,
        order_by: &str,
    ) -> Result<PaginatedProductDailyPriceComparison, AppError>;

    /// Aprox price by auditor user
    /// This method is batch operation
    async fn approve_price(
        &self,
        apprived_by: &str,
        approval_params: &PriceApprovalParam,
    ) -> Result<(), AppError>;

    async fn batch_create_product_price(
        &self,
        username: &str,
        product_prices: &[PriceCreateDTO],
    ) -> Result<u64, AppError>;
}

#[async_trait]
impl PriceRepository for MySqlRepository {
    async fn list_price_announcements_by_category_product_name_and_date(
        &self,
        params: &PriceQueryParams,
    ) -> Result<Vec<PriceAnnouncement>, AppError> {
        debug!("get_published_prices: {:?}", params);

        // 检查是否是今天且当前时间是否在中午12点之前
        let now = Local::now();
        let current_time = now.time();
        let noon = NaiveTime::from_hms_opt(12, 0, 0).unwrap();

        // 如果是今天且在中午12点之前，我们应该考虑昨天的价格变动
        let consider_previous_day = current_time < noon && params.date.is_none();

        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(
            r#"SELECT 
            c1.name as level1_category,
            c3.name as level3_category,
            p.name as product_name,
            p.product_code,
            p.unit,
            COALESCE(latest_price.min_price, 0) as min_price,
            CASE "#,
        );

        if consider_previous_day {
            builder.push(r#"
                WHEN latest_price.price_date = CURRENT_DATE THEN COALESCE(latest_price.min_price_change, 0)
                WHEN latest_price.price_date = DATE_SUB(CURRENT_DATE, INTERVAL 1 DAY) THEN COALESCE(latest_price.min_price_change, 0)
                ELSE 0
            "#);
        } else {
            builder.push(" WHEN latest_price.price_date = ");

            if let Some(date) = &params.date {
                builder.push_bind(date);
            } else {
                builder.push("CURRENT_DATE");
            }

            builder.push(" THEN COALESCE(latest_price.min_price_change, 0) ELSE 0 ");
        }

        builder.push(
            r#"END as min_price_change,
            COALESCE(latest_price.avg_price, 0) as avg_price,
            CASE "#,
        );

        if consider_previous_day {
            builder.push(r#"
                WHEN latest_price.price_date = CURRENT_DATE THEN COALESCE(latest_price.avg_price_change, 0)
                WHEN latest_price.price_date = DATE_SUB(CURRENT_DATE, INTERVAL 1 DAY) THEN COALESCE(latest_price.avg_price_change, 0)
                ELSE 0
            "#);
        } else {
            builder.push(" WHEN latest_price.price_date = ");

            if let Some(date) = &params.date {
                builder.push_bind(date);
            } else {
                builder.push("CURRENT_DATE");
            }

            builder.push(" THEN COALESCE(latest_price.avg_price_change, 0) ELSE 0 ");
        }

        builder.push(
            r#"END as avg_price_change,
            COALESCE(latest_price.max_price, 0) as max_price,
            CASE "#,
        );

        if consider_previous_day {
            builder.push(r#"
                WHEN latest_price.price_date = CURRENT_DATE THEN COALESCE(latest_price.max_price_change, 0)
                WHEN latest_price.price_date = DATE_SUB(CURRENT_DATE, INTERVAL 1 DAY) THEN COALESCE(latest_price.max_price_change, 0)
                ELSE 0
            "#);
        } else {
            builder.push(" WHEN latest_price.price_date = ");

            if let Some(date) = &params.date {
                builder.push_bind(date);
            } else {
                builder.push("CURRENT_DATE");
            }

            builder.push(" THEN COALESCE(latest_price.max_price_change, 0) ELSE 0 ");
        }

        builder.push(
            r#"END as max_price_change,
            CASE 
                WHEN latest_price.id IS NULL THEN 'NOT_PUBLISHED'
                ELSE 'PUBLISHED'
            END as price_status,
            latest_price.price_date
        FROM products p
        JOIN categories c3 ON p.category_id = c3.id AND c3.level = 3
        JOIN categories c2 ON c2.id = c3.parent_id AND c2.level = 2
        JOIN categories c1 ON c1.id = c2.parent_id AND c1.level = 1
        LEFT JOIN (
            SELECT pp1.*
            FROM product_prices pp1
            JOIN (
                SELECT product_id, 
                    CASE 
                        WHEN EXISTS (
                            SELECT 1 FROM product_prices pp_inner 
                            WHERE pp_inner.product_id = p_outer.product_id 
                            AND pp_inner.price_date = "#,
        );

        if let Some(date) = &params.date {
            builder.push_bind(date);
        } else {
            builder.push("CURRENT_DATE");
        }

        builder.push(
            r#" AND pp_inner.status = 'PUBLISHED'
                        ) THEN "#,
        );

        if let Some(date) = &params.date {
            builder.push_bind(date);
        } else {
            builder.push("CURRENT_DATE");
        }

        builder.push(
            r#"
                        ELSE (
                            SELECT MAX(price_date) 
                            FROM product_prices 
                            WHERE product_id = p_outer.product_id 
                            AND status = 'PUBLISHED'
                            AND price_date <= "#,
        );

        if let Some(date) = &params.date {
            builder.push_bind(date);
        } else {
            builder.push("CURRENT_DATE");
        }

        builder.push(
            r#"
                        )
                    END as latest_date
                FROM product_prices p_outer
                GROUP BY product_id
            ) latest ON pp1.product_id = latest.product_id 
            AND pp1.price_date = latest.latest_date
            AND pp1.status = 'PUBLISHED'
        ) latest_price ON p.id = latest_price.product_id
        WHERE 1=1"#,
        );

        if let Some(name) = &params.name {
            builder
                .push(" AND p.name LIKE ")
                .push_bind(format!("%{}%", name));
        } else {
            if let Some(category1) = &params.category_l1 {
                builder.push(" AND c1.id = ").push_bind(category1);
            } else {
                builder.push(" AND c1.id = 759 ");
            }
            if let Some(category3) = &params.category_l3 {
                builder.push(" AND c3.id = ").push_bind(category3);
            }
        }

        builder.push(" AND p.is_disabled = 0 ORDER BY c1.name, c3.name, p.name");

        builder
            .build_query_as::<PriceAnnouncement>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get product prices"))
    }

    /// Get daily price comparison for PRICER users
    ///
    /// Retrieves products associated with the PRICER user's assigned categories
    /// that do not have any price records for the current day, along with their
    /// most recent PUBLISHED prices as reference. This helps PRICER users identify
    /// products that need to be priced for the current day.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the PRICER user
    /// * `role_name` - The role name (should be "PRICER")
    /// * `query` - Pagination parameters including page number and page size
    ///
    /// # Returns
    ///
    /// Returns a list containing:
    /// - Products without current day prices
    /// - Historical PUBLISHED prices as reference (yesterday_min_price, yesterday_avg_price, yesterday_max_price)
    /// - Current day prices set to 0.00 (since no pricing exists yet)
    /// - Status set to "HISTORY" indicating these are historical references
    ///
    /// # Errors
    ///
    /// Returns an `AppError` if:
    /// - Database query fails (connection, syntax, etc.)
    /// - User has no category assignments
    /// - Invalid role name is provided
    /// - Pagination parameters are invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// let query = PriceStatusPaginationParams {
    ///     page: Some(1),
    ///     page_size: Some(20),
    ///     ..Default::default()
    /// };
    /// let result = repo.list_pricer_daily_price_comparison(
    ///     "pricer_user",
    ///     "PRICER",
    ///     &query
    /// ).await?;
    /// ```
    async fn list_pricer_daily_price_comparison(
        &self,
        username: &str,
        role_name: &str,
        query: &PriceStatusPaginationParams,
    ) -> Result<PaginatedProductDailyPriceComparison, AppError> {
        debug!("Querying unpriced products for PRICER: {}", username);

        let select_fields = r#"p.id AS product_id,
        p.name AS product_name,
        p.unit,
        c.name AS assigned_category_name,
        COALESCE(latest_pp.min_price, CAST(0 AS DECIMAL(10,2))) AS yesterday_min_price,
        COALESCE(latest_pp.avg_price, CAST(0 AS DECIMAL(10,2))) AS yesterday_avg_price,
        COALESCE(latest_pp.max_price, CAST(0 AS DECIMAL(10,2))) AS yesterday_max_price,
        CAST(0 AS DECIMAL(10,2)) AS today_min_price,
        CAST(0 AS DECIMAL(10,2)) AS today_avg_price,
        CAST(0 AS DECIMAL(10,2)) AS today_max_price,
        'HISTORY' AS price_status,
        'HISTORY' AS price_source,
        latest_pp.price_date AS publish_date"#;

        let additional_joins = r#"LEFT JOIN (
            -- 获取产品最近的PUBLISHED状态价格(排除当日)
            SELECT pp.*
            FROM product_prices pp
            JOIN (
                SELECT product_id, MAX(price_date) as max_date
                FROM product_prices
                WHERE status = 'PUBLISHED'
                AND price_date < CURRENT_DATE
                GROUP BY product_id
            ) latest ON pp.product_id = latest.product_id AND pp.price_date = latest.max_date
            WHERE pp.status = 'PUBLISHED'
        ) AS latest_pp ON p.id = latest_pp.product_id"#;

        let where_clause = r#"NOT EXISTS (
            SELECT 1
            FROM product_prices today_pp
            WHERE today_pp.product_id = p.id
            AND today_pp.price_date = CURRENT_DATE
        )"#;

        let order_by = "c.sort_order, p.id DESC";

        self.query_prices_paginated(
            username,
            role_name,
            query,
            where_clause,
            select_fields,
            additional_joins,
            order_by,
        )
        .await
    }

    async fn list_price_products_by_status(
        &self,
        username: &str,
        role_name: &str,
        query: &PriceStatusPaginationParams,
    ) -> Result<PaginatedProductDailyPriceComparison, AppError> {
        // 确定要查询的状态，默认为PENDING
        let target_status = if let Some(status) = &query.status {
            match status.as_str() {
                "PENDING" | "REJECTED" | "PUBLISHED" | "APPROVED" => status,
                _ => {
                    return Err(AppError::validation(
                        "Invalid status. Must be one of: PENDING, REJECTED, PUBLISHED",
                    ))
                }
            }
        } else {
            "PENDING"
        };

        debug!(
            "Querying {} status products for user: {}, username: {}",
            target_status, role_name, username
        );

        let select_fields = r#"p.id AS product_id,
        p.name AS product_name,
        p.unit,
        c.name AS assigned_category_name,
        COALESCE(latest_pp.min_price, CAST(0 AS DECIMAL(10,2))) AS yesterday_min_price,
        COALESCE(latest_pp.avg_price, CAST(0 AS DECIMAL(10,2))) AS yesterday_avg_price,
        COALESCE(latest_pp.max_price, CAST(0 AS DECIMAL(10,2))) AS yesterday_max_price,
        pp.min_price AS today_min_price,
        pp.avg_price AS today_avg_price,
        pp.max_price AS today_max_price,
        pp.status AS price_status,
        'TODAY' AS price_source,
        pp.price_date AS publish_date"#;

        let additional_joins = r#"JOIN product_prices pp ON p.id = pp.product_id
        LEFT JOIN (
            -- 获取产品最近的PUBLISHED状态价格（不包括当天）
            SELECT pp.*
            FROM product_prices pp
            JOIN (
                SELECT product_id, MAX(price_date) as max_date
                FROM product_prices
                WHERE status = 'PUBLISHED'
                AND price_date < CURRENT_DATE
                GROUP BY product_id
            ) latest ON pp.product_id = latest.product_id AND pp.price_date = latest.max_date
            WHERE pp.status = 'PUBLISHED'
        ) AS latest_pp ON p.id = latest_pp.product_id"#;

        let where_clause = r#"pp.price_date = CURRENT_DATE
        AND pp.status = ?"#;

        let order_by = "c.sort_order, p.id DESC";

        // Since we need to bind the status, we'll need to create a custom implementation
        // that handles the status parameter binding
        let page = query.page.unwrap_or(1) as u32;
        let page_size = query.page_size.unwrap_or(50) as u32;
        let offset = (page - 1) * page_size;

        // 根据角色决定是否使用用户分类关联
        let is_pricer_role = role_name == "PRICER";

        let (from_clause, category_join) = if is_pricer_role {
            // PRICER角色：使用用户分类关联
            let category_join = r#"
            JOIN (
            -- 为一级分类找到所有关联的三级分类
            SELECT c3.id, c1.id AS root_id
            FROM categories c1
            JOIN categories c2 ON c2.parent_id = c1.id AND c2.level = 2
            JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
            WHERE c1.level = 1

            UNION ALL

            -- 为二级分类找到所有关联的三级分类
            SELECT c3.id, c2.id AS root_id
            FROM categories c2
            JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
            WHERE c2.level = 2

            UNION ALL

            -- 三级分类直接关联自己
            SELECT c3.id, c3.id AS root_id
            FROM categories c3
            WHERE c3.level = 3
            ) AS category_mapping ON (
            c.id = category_mapping.root_id
            )
            JOIN
            products p ON p.category_id = category_mapping.id"#;

            let from_clause = r#"FROM
            users u
            JOIN
            user_roles ur ON u.id = ur.user_id AND ur.role_id = (SELECT id FROM roles WHERE name = ? LIMIT 1)
            JOIN
            user_category_assignments uca ON u.id = uca.user_id
            JOIN
            categories c ON uca.category_id = c.id"#;

            (from_clause, category_join)
        } else {
            // 非PRICER角色（如AUDITOR）：直接查询所有产品，不使用用户分类关联
            let category_join = r#"
            JOIN
            products p ON p.category_id = c.id"#;

            let from_clause = r#"FROM
            users u
            JOIN
            user_roles ur ON u.id = ur.user_id AND ur.role_id = (SELECT id FROM roles WHERE name = ? LIMIT 1)
            CROSS JOIN
            categories c1
            JOIN categories c2 ON c2.parent_id = c1.id AND c2.level = 2
            JOIN categories c ON c.parent_id = c2.id AND c.level = 3"#;

            (from_clause, category_join)
        };

        // 构建count查询
        let mut count_sql = format!(
            r#"SELECT COUNT(*) as total
        {} {} {}"#,
            from_clause, category_join, additional_joins
        );

        if is_pricer_role {
            count_sql.push_str(" WHERE u.username = ? AND ");
        } else {
            count_sql.push_str(" WHERE u.username = ? AND ");
        }
        count_sql.push_str(where_clause);

        debug!(
            "Counting products for user: {}, username: {}, where: {}, is_pricer: {}",
            role_name, username, where_clause, is_pricer_role
        );

        let total: i64 = if is_pricer_role {
            sqlx::query_scalar(&count_sql)
                .bind(role_name)
                .bind(username)
                .bind(target_status)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_err!("Failed to count products"))?
        } else {
            sqlx::query_scalar(&count_sql)
                .bind(role_name)
                .bind(username)
                .bind(target_status)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_err!("Failed to count products"))?
        };

        // 如果总数为0，直接返回空结果
        if total == 0 {
            return Ok(PaginatedProductDailyPriceComparison {
                data: vec![],
                total: 0,
                page,
                page_size,
            });
        }

        // 构建数据查询
        let data_sql = format!(
            r#"SELECT {}
        {} {} {}"#,
            select_fields, from_clause, category_join, additional_joins
        );

        let mut final_query = data_sql;
        if is_pricer_role {
            final_query.push_str(" WHERE u.username = ? AND ");
        } else {
            final_query.push_str(" WHERE u.username = ? AND ");
        }
        final_query.push_str(where_clause);
        final_query.push_str(" ORDER BY ");
        final_query.push_str(order_by);
        final_query.push_str(" LIMIT ? OFFSET ?");

        // debug!("Querying products for user: {}, username: {}, SQL: {}", role_name, username, &final_query);

        let product_prices = if is_pricer_role {
            sqlx::query_as::<_, ProductDailyPriceComparisonDTO>(&final_query)
                .bind(role_name)
                .bind(username)
                .bind(target_status)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get products"))?
        } else {
            sqlx::query_as::<_, ProductDailyPriceComparisonDTO>(&final_query)
                .bind(role_name)
                .bind(username)
                .bind(target_status)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get products"))?
        };

        Ok(PaginatedProductDailyPriceComparison {
            data: product_prices,
            total: total as u64,
            page,
            page_size,
        })
    }

    /// 通用辅助函数：执行用户产品分页查询
    ///
    /// # Arguments
    /// * `username` - 用户名
    /// * `role_name` - 用户角色名
    /// * `query` - 分页查询参数
    /// * `where_clause` - WHERE条件子句（不包含WHERE关键字）
    /// * `select_fields` - SELECT字段列表
    /// * `additional_joins` - 额外的JOIN条件
    /// * `order_by` - ORDER BY子句
    ///
    /// # Returns
    /// * `PaginatedProductDailyPriceComparison` - 分页结果
    async fn query_prices_paginated(
        &self,
        username: &str,
        role_name: &str,
        query: &PriceStatusPaginationParams,
        where_clause: &str,
        select_fields: &str,
        additional_joins: &str,
        order_by: &str,
    ) -> Result<PaginatedProductDailyPriceComparison, AppError> {
        let page = query.page.unwrap_or(1) as u32;
        let page_size = query.page_size.unwrap_or(50) as u32;
        let offset = (page - 1) * page_size;

        // 通用的用户分类关联部分
        let category_join = r#"
        JOIN (
        -- 为一级分类找到所有关联的三级分类
        SELECT c3.id, c1.id AS root_id
        FROM categories c1
        JOIN categories c2 ON c2.parent_id = c1.id AND c2.level = 2
        JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
        WHERE c1.level = 1

        UNION ALL

        -- 为二级分类找到所有关联的三级分类
        SELECT c3.id, c2.id AS root_id
        FROM categories c2
        JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
        WHERE c2.level = 2

        UNION ALL

        -- 三级分类直接关联自己
        SELECT c3.id, c3.id AS root_id
        FROM categories c3
        WHERE c3.level = 3
        ) AS category_mapping ON (
        c.id = category_mapping.root_id
        )
        JOIN
        products p ON p.category_id = category_mapping.id"#;

        // 构建count查询
        let mut count_sql = format!(
            r#"SELECT COUNT(*) as total
        FROM
        users u
        JOIN
        user_roles ur ON u.id = ur.user_id AND ur.role_id = (SELECT id FROM roles WHERE name = ? LIMIT 1)
        JOIN
        user_category_assignments uca ON u.id = uca.user_id
        JOIN
        categories c ON uca.category_id = c.id
        {} {}"#,
            category_join, additional_joins
        );

        count_sql.push_str(" WHERE u.username = ?");
        if !where_clause.is_empty() {
            count_sql.push_str(" AND ");
            count_sql.push_str(where_clause);
        }

        debug!(
            "Counting products for user: {}, username: {}, where: {}",
            role_name, username, where_clause
        );

        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(role_name)
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to count products"))?;

        // 如果总数为0，直接返回空结果
        if total == 0 {
            return Ok(PaginatedProductDailyPriceComparison {
                data: vec![],
                total: 0,
                page,
                page_size,
            });
        }

        // 构建数据查询
        let data_sql = format!(
            r#"SELECT {}
        FROM
        users u
        JOIN
        user_roles ur ON u.id = ur.user_id AND ur.role_id = (SELECT id FROM roles WHERE name = ? LIMIT 1)
        JOIN
        user_category_assignments uca ON u.id = uca.user_id
        JOIN
        categories c ON uca.category_id = c.id
        {} {}"#,
            select_fields, category_join, additional_joins
        );

        let mut final_query = data_sql;
        final_query.push_str(" WHERE u.username = ?");
        if !where_clause.is_empty() {
            final_query.push_str(" AND ");
            final_query.push_str(where_clause);
        }
        final_query.push_str(" ORDER BY ");
        final_query.push_str(order_by);
        final_query.push_str(" LIMIT ? OFFSET ?");

        debug!(
            "Querying products for user: {}, username: {}, SQL: {}",
            role_name, username, &final_query
        );

        let product_prices = sqlx::query_as::<_, ProductDailyPriceComparisonDTO>(&final_query)
            .bind(role_name)
            .bind(username)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get products"))?;

        Ok(PaginatedProductDailyPriceComparison {
            data: product_prices,
            total: total as u64,
            page,
            page_size,
        })
    }

    async fn approve_price(
        &self,
        approved_by: &str,
        approval_params: &PriceApprovalParam,
    ) -> Result<(), AppError> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Faild to begin transaction"))?;

        let mut builder = QueryBuilder::new("UPDATE product_prices SET status = ");

        let status = approval_params.status.to_uppercase();
        builder.push_bind(status);
        builder.push(" ,approved_by = ").push_bind(approved_by);
        builder.push(" ,approved_at = CURRENT_TIMESTAMP ");

        if approval_params.remark.is_some() {
            builder
                .push(", remark = ")
                .push_bind(approval_params.remark.as_ref());
        }
        builder.push(" WHERE price_date = CURRENT_DATE AND product_id IN (");

        let mut separated = builder.separated(", ");
        for product_id in &approval_params.products {
            separated.push_bind(product_id);
        }

        separated.push_unseparated(");");

        debug!("Row sql: {}", builder.sql());

        builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Failed to update product price status"))?;
        debug!("Row sql: {}", builder.sql());
        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(())
    }

    async fn batch_create_product_price(
        &self,
        username: &str,
        product_prices: &[PriceCreateDTO],
    ) -> Result<u64, AppError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(map_db_err!("Failed to begin transaction"))?;

        // TODO: 检查产品是否存在或状态是否为REJECTED，如果存在并状态为REJECTED，则不进行插入
        // 将原状态更改为PENDING，其他状态则不允许插入和更改

        // 获取用户信息和市场ID
        let user_info = sqlx::query!(
            r#"
            select u.name as name, t.id as market_id from users u INNER join tenants t on u.tenant_id = t.id where u.username=?;
            "#,
            username
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_err!("Failed to get user info"))?;

        let created_by = user_info.name;
        let market_id = user_info.market_id;

        // 准备批量插入语句
        let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
            "INSERT INTO product_prices (product_id, min_price, max_price, avg_price, min_price_change, max_price_change, avg_price_change, market_id, created_by) "
        );

        query_builder.push_values(product_prices, |mut b, price| {
            b.push_bind(price.product_id)
                .push_bind(price.min_price)
                .push_bind(price.max_price)
                .push_bind(price.avg_price)
                .push_bind(price.min_price_change)
                .push_bind(price.max_price_change)
                .push_bind(price.avg_price_change)
                .push_bind(market_id)
                .push_bind(&created_by);
        });

        // 如果存在则更新，否则插入，并设置状态为PENDING
        // 在业务流程上，产品数据第一次提交后除了在REJECTED状态外，其他状态在前端都设置为不编辑不可提交
        // 所以这进行更新操作，意味着询价员对REJECTED状态的产品进行了修改
        query_builder.push(
            " ON DUPLICATE KEY UPDATE 
            min_price = VALUES(min_price),
            max_price = VALUES(max_price),
            avg_price = VALUES(avg_price),
            min_price_change = VALUES(min_price_change),
            max_price_change = VALUES(max_price_change),
            avg_price_change = VALUES(avg_price_change),
            updated_at = NOW(),
            status = 'PENDING'",
        );

        // 执行查询
        let result = query_builder.build().execute(&mut *tx).await?;
        tx.commit()
            .await
            .map_err(map_db_err!("Failed to commit transaction"))?;
        Ok(result.rows_affected())
    }
}
