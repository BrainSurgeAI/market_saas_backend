use super::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::price::{AproxPriceParam, PriceAnnouncement, PriceQueryParams},
    map_db_err,
};
use async_trait::async_trait;
use chrono::{Local, NaiveTime};
use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error};

#[async_trait]
pub trait PriceRepository: Send + Sync {

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
    async fn aprox_price(&self, query: &AproxPriceParam) -> Result<(), AppError>;
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
}
