use super::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::price::{
        AproxPriceParam, 
        PriceAnnouncement, 
        PriceQueryParams, 
        PriceStatusPaginationParams, 
        PriceCreateDTO,
    },
    dto::products::ProductDailyPriceComparisonDTO,
    map_db_err,
};
use async_trait::async_trait;
use chrono::{Local, NaiveTime};
use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error};

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

    async fn find_price_announcements_by_category_product_name_and_date(
        &self,
        query: &PriceQueryParams,
    ) -> Result<Vec<PriceAnnouncement>, AppError>;


     /// Find product daily price comparison by user with role-based filtering.
    ///
    /// This function retrieves a comprehensive comparison of product prices for a specific user,
    /// showing both today's and yesterday's pricing data with role-based access control.
    /// It handles complex category hierarchies and provides intelligent price source prioritization.
    ///
    /// # Parameters
    ///
    /// * `username` - The username of the user requesting the price comparison
    /// * `role_name` - The role name of the user (e.g., "AUDITOR", "PRICER") which determines data access
    /// * `query` - Query parameters containing optional status filtering for AUDITOR role
    ///
    /// # Returns
    ///
    /// A vector of [`ProductDailyPriceComparisonDTO`] containing:
    /// - Product基本信息 (ID, name, unit, assigned category)
    /// - 昨天价格 (min, avg, max) - 从历史数据获取
    /// - 今天价格 (min, avg, max) - 根据状态决定显示
    /// - 价格状态 (PENDING, APPROVED, PUBLISHED, REJECTED)
    /// - 价格来源 (TODAY, HISTORY)
    /// - 发布日期
    ///
    /// # Role-Based Behavior
    ///
    /// **AUDITOR Role:**
    /// - Can filter by specific status (PENDING, APPROVED, PUBLISHED, REJECTED)
    /// - Sees all price entries for the current date
    /// - Used for auditing and price review purposes
    ///
    /// **PRICER Role:**
    /// - Only sees PUBLISHED status prices from history
    /// - Today's prices default to PENDING status
    /// - Used for price entry and management
    ///
    /// # Price Source Logic
    ///
    /// The query implements sophisticated price source prioritization:
    ///
    /// 1. **Today's Prices (TODAY source):**
    ///    - When `price_date = CURRENT_DATE`
    ///    - Shows all statuses including REJECTED
    ///    - Displayed as today's min/avg/max prices
    ///
    /// 2. **Historical Prices (HISTORY source):**
    ///    - When `price_date < CURRENT_DATE`
    ///    - Only PUBLISHED status from recent historical data
    ///    - Used as fallback when no today's data exists
    ///    - Displayed as yesterday's min/avg/max prices when today's source is HISTORY
    ///
    /// 3. **Yesterday's Prices Calculation:**
    ///    - If today's source is HISTORY: show today's data as "yesterday's"
    ///    - If today's source is TODAY: query actual yesterday's PUBLISHED data
    ///    - Handles the transition from pending to approved to published pricing
    ///
    /// # Category Hierarchy Handling
    ///
    /// The query supports complex category assignments:
    ///
    /// **Level 1 Assignment:**
    /// - User assigned to Level 1 category sees all Level 2 and Level 3 subcategories
    /// - Example: User assigned to "熟食卤味" sees all subcategories
    ///
    /// **Level 2 Assignment:**
    /// - User assigned to Level 2 category sees all Level 3 subcategories
    /// - Example: User assigned to "猪肉卤制品" sees specific product categories
    ///
    /// **Level 3 Assignment:**
    /// - User assigned to Level 3 category sees only products in that category
    /// - Most granular level of access control
    ///
    /// # SQL Query Structure
    ///
    /// The complex SQL query consists of:
    /// - **User & Category Joins:** Links users to their assigned categories
    /// - **Category Mapping Subquery:** Resolves category hierarchy relationships
    /// - **Price Logic Subquery:** Implements price source prioritization
    /// - **Conditional Filtering:** Role-based WHERE clauses
    ///
    /// # Examples
    ///
    /// ```rust
    /// // AUDITOR can see specific status
    /// let query = QueryPriceByStatusParams {
    ///     status: Some("PENDING".to_string()),
    /// };
    /// let prices = repo.find_product_daily_price_comparison_by_user(
    ///     "auditor_user",
    ///     "AUDITOR",
    ///     &query
    /// ).await?;
    ///
    /// // PRICER sees PUBLISHED historical data
    /// let query = QueryPriceByStatusParams { status: None };
    /// let prices = repo.find_product_daily_price_comparison_by_user(
    ///     "pricer_user",
    ///     "PRICER",
    ///     &query
    /// ).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an `AppError` if:
    /// - Database query fails (connection, syntax, etc.)
    /// - User has no category assignments
    /// - Invalid role name is provided
    async fn user_daily_price_comparison(
        &self,
        username: &str,
        role_name: &str,
        query: &PriceStatusPaginationParams,
    ) -> Result<Vec<ProductDailyPriceComparisonDTO>, AppError>;


    async fn aprox_price(&self, query: &AproxPriceParam) -> Result<(), AppError>;

    async fn batch_create_product_price(
        &self,
        username: &str,
        product_prices: &[PriceCreateDTO],
    ) -> Result<u64, AppError>;
}

#[async_trait]
impl PriceRepository for MySqlRepository {

    async fn find_price_announcements_by_category_product_name_and_date(
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


    async fn user_daily_price_comparison(
        &self,
        username: &str,
        role_name: &str,
        query: &PriceStatusPaginationParams,
    ) -> Result<Vec<ProductDailyPriceComparisonDTO>, AppError> {
        let sql = r#"SELECT 
        p.id AS product_id,
        p.name AS product_name,
        p.unit,
        c.name AS assigned_category_name,
        COALESCE(
            CASE 
                WHEN today_pp.price_source = 'HISTORY' THEN today_pp.min_price
                ELSE yesterday_pp.min_price 
            END, 0
        ) AS yesterday_min_price,
        COALESCE(
            CASE 
                WHEN today_pp.price_source = 'HISTORY' THEN today_pp.avg_price
                ELSE yesterday_pp.avg_price 
            END, 0
        ) AS yesterday_avg_price,
        COALESCE(
            CASE 
                WHEN today_pp.price_source = 'HISTORY' THEN today_pp.max_price
                ELSE yesterday_pp.max_price 
            END, 0
        ) AS yesterday_max_price,
        COALESCE(
            CASE 
                WHEN today_pp.price_source = 'TODAY' THEN today_pp.min_price
                ELSE 0
            END, 0
        ) AS today_min_price,
        COALESCE(
            CASE 
                WHEN today_pp.price_source = 'TODAY' THEN today_pp.avg_price
                ELSE 0
            END, 0
        ) AS today_avg_price,
        COALESCE(
            CASE 
                WHEN today_pp.price_source = 'TODAY' THEN today_pp.max_price
                ELSE 0
            END, 0
        ) AS today_max_price,
        today_pp.status AS price_status,
        today_pp.price_source AS price_source,
        today_pp.price_date AS publish_date
        FROM 
        users u
        JOIN 
        user_roles ur ON u.id = ur.user_id AND ur.role_id = (SELECT id FROM roles WHERE name = ? LIMIT 1)
        JOIN 
        user_category_assignments uca ON u.id = uca.user_id
        JOIN 
        categories c ON uca.category_id = c.id
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
        products p ON p.category_id = category_mapping.id
        LEFT JOIN 
        product_prices yesterday_pp ON p.id = yesterday_pp.product_id 
        AND yesterday_pp.status = 'PUBLISHED' 
        AND yesterday_pp.price_date = DATE_SUB(CURRENT_DATE, INTERVAL 1 DAY)
        LEFT JOIN (
        -- 获取价格信息，优先显示当日价格（包括REJECTED状态），如果没有当日价格则显示最近的已发布价格
        SELECT pp1.*,
        CASE 
            WHEN pp1.price_date = CURRENT_DATE THEN 'TODAY' 
            ELSE 'HISTORY' 
        END AS price_source
        FROM product_prices pp1
        JOIN (
            SELECT product_id, 
                CASE 
                    -- 如果存在当日价格（包括REJECTED状态），则使用当日
                    WHEN EXISTS (
                        SELECT 1 FROM product_prices pp_inner 
                        WHERE pp_inner.product_id = p_outer.product_id 
                        AND pp_inner.price_date = CURRENT_DATE
                    ) THEN CURRENT_DATE
                    -- 否则使用最近的已发布价格日期
                    ELSE (
                        SELECT MAX(price_date) 
                        FROM product_prices 
                        WHERE product_id = p_outer.product_id 
                        AND status = ?
                        AND price_date <= CURRENT_DATE
                    )
                END as latest_date
            FROM product_prices p_outer
            GROUP BY product_id
        ) latest ON pp1.product_id = latest.product_id 
        AND pp1.price_date = latest.latest_date
        ) AS today_pp ON p.id = today_pp.product_id
        WHERE 
        u.username = ?"#;

        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(sql);

        if role_name == "AUDITOR" {
            if let Some(status) = &query.status {
                builder.push(" AND EXISTS (SELECT 1 FROM product_prices WHERE product_id = p.id AND price_date = CURRENT_DATE AND status = '");
                builder.push(status);
                builder.push("') ");
            }
        }

        builder.push(" ORDER BY c.sort_order, p.id DESC LIMIT ? OFFSET ?;");

        let status = if role_name == "PRICER" {
            "PUBLISHED"
        } else {
            "PENDING"
        };

        let product_prices = builder
            .build_query_as::<ProductDailyPriceComparisonDTO>()
            .bind(role_name)
            .bind(status)
            .bind(username)
            .bind(query.page_size.unwrap())
            .bind(query.page.unwrap() - 1)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get product prices"))?;
        Ok(product_prices)
    }

    async fn aprox_price(&self, query: &AproxPriceParam) -> Result<(), AppError> {
        let mut builder = QueryBuilder::new("UPDATE product_prices SET status = ");

        builder.push_bind(&query.status);
        builder.push(" WHERE price_date = CURRENT_DATE AND product_id IN (");

        let mut separated = builder.separated(", ");
        for product_id in &query.products {
            separated.push_bind(product_id);
        }

        separated.push_unseparated(")");

        builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Failed to update product price status"))?;

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
