use super::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::{
        category::CategoryDTO,
        price::QueryPriceByStatusParams,
        products::{
            PriceStatusDTO, ProcessingFeeDTO, ProductDailyPriceComparisonDTO, ProductDetailDTO,
            ProductDetailResponse, ProductListDTO, ProductListQueryParams, ProductOverviewDTO,
            ProductPriceCreateDTO, UpdateProductRequestDTO,
        },
    },
    map_db_err,
};
use async_trait::async_trait;
use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error};

#[async_trait]
pub trait ProductRepository: Send + Sync {
    // 根据分类ID获取产品列表，返回产品列表，用于产品管理
    async fn fetch_products_by_category_id(
        &self,
        query: &ProductListQueryParams,
    ) -> Result<(Vec<ProductOverviewDTO>, i32), AppError>;

    async fn get_level1_categories(&self) -> Result<Vec<CategoryDTO>, AppError>;

    async fn find_product_daily_price_comparison_by_user(
        &self,
        username: &str,
        role_name: &str,
        query: &QueryPriceByStatusParams,
    ) -> Result<Vec<ProductDailyPriceComparisonDTO>, AppError>;

    async fn batch_create_product_price(
        &self,
        username: &str,
        product_prices: &[ProductPriceCreateDTO],
    ) -> Result<u64, AppError>;

    // 获取当日价格状态统计
    async fn fetch_product_price_status_stats(
        &self,
        username: &str,
        role_name: &str,
    ) -> Result<Vec<PriceStatusDTO>, AppError>;

    async fn get_product_list(
        &self,
        query: &ProductListQueryParams,
        tenant_hash: &str,
    ) -> Result<Vec<ProductListDTO>, AppError>;

    async fn get_product_detail(
        &self,
        product_code: &str,
        is_owner: bool,
    ) -> Result<Option<ProductDetailResponse>, AppError>;

    async fn deactivate_product(&self, product_code: &str) -> Result<(), AppError>;

    async fn update_product(
        &self,
        product_code: &str,
        product: &UpdateProductRequestDTO,
    ) -> Result<(), AppError>;

    async fn get_processing_fees(&self) -> Result<Vec<ProcessingFeeDTO>, AppError>;
}

#[async_trait]
impl ProductRepository for MySqlRepository {
    // 市场管理员获取产品列表
    async fn fetch_products_by_category_id(
        &self,
        query: &ProductListQueryParams,
    ) -> Result<(Vec<ProductOverviewDTO>, i32), AppError> {
        debug!("fetch_products_by_category_id: {:?}", query);
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(12);
        let offset = (page - 1) * page_size;

        // 基础SQL查询 - 用于数据查询
        let base_sql = r#"
            SELECT
                p.product_code,
                p.name,
                p.unit,
                p.product_description AS description,
                p.is_disabled
            FROM 
                products p
            JOIN 
                categories c3 ON p.category_id = c3.id AND c3.level = 3
            JOIN 
                categories c2 ON c3.parent_id = c2.id AND c2.level = 2
            JOIN 
                categories c1 ON c2.parent_id = c1.id AND c1.level = 1
            WHERE 1=1
        "#;

        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(base_sql);

        // 添加筛选条件
        if let Some(category_id) = query.category_id {
            builder.push(" AND c1.id = ").push_bind(category_id);
        }

        if let Some(name) = &query.name {
            builder
                .push(" AND p.name LIKE ")
                .push_bind(format!("%{}%", name));
        }

        // 添加排序
        builder.push(" ORDER BY p.name");

        // 添加分页
        builder.push(" LIMIT ").push_bind(page_size);
        builder.push(" OFFSET ").push_bind(offset);

        // 执行分页查询
        let products = builder
            .build_query_as::<ProductOverviewDTO>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to fetch products by category"))?;

        // 获取总记录数
        let total_count: i32 = if let Some(category_id) = query.category_id {
            // 使用COUNT查询获取总数
            let count_sql = r#"
                SELECT COUNT(*) AS total_count
                FROM products p
                JOIN categories c3 ON p.category_id = c3.id AND c3.level = 3
                JOIN categories c2 ON c3.parent_id = c2.id AND c2.level = 2
                JOIN categories c1 ON c2.parent_id = c1.id AND c1.level = 1
                WHERE 1=1
            "#;

            let mut count_builder: QueryBuilder<MySql> = QueryBuilder::new(count_sql);

            count_builder.push(" AND c1.id = ").push_bind(category_id);

            if let Some(name) = &query.name {
                count_builder
                    .push(" AND p.name LIKE ")
                    .push_bind(format!("%{}%", name));
            }

            count_builder
                .build_query_scalar::<i32>()
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_err!("Failed to count products by category"))?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM products")
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_err!("Failed to count products"))?
        };

        debug!("Total count: {}", total_count);
        Ok((products, total_count))
    }

    async fn get_level1_categories(&self) -> Result<Vec<CategoryDTO>, AppError> {
        let sql = "SELECT id, name as level1_category FROM categories WHERE level = 1 ORDER BY sort_order DESC";
        let categories = sqlx::query_as::<_, CategoryDTO>(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get level 1 categories"))?;
        Ok(categories)
    }

    async fn find_product_daily_price_comparison_by_user(
        &self,
        username: &str,
        role_name: &str,
        query: &QueryPriceByStatusParams,
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

        builder.push(" ORDER BY c.sort_order, c.name, p.name DESC;");

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
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get product prices"))?;
        Ok(product_prices)
    }

    async fn batch_create_product_price(
        &self,
        username: &str,
        product_prices: &[ProductPriceCreateDTO],
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

    // 获取当日价格状态统计
    async fn fetch_product_price_status_stats(
        &self,
        username: &str,
        role_name: &str,
    ) -> Result<Vec<PriceStatusDTO>, AppError> {
        let sql = r#"
        SELECT
    c.id AS category_id,
    c.name AS category_name,
  
    COUNT(DISTINCT p.id) AS total_products,
    COUNT(DISTINCT CASE WHEN NOT EXISTS (
        SELECT 1 FROM product_prices pp 
        WHERE pp.product_id = p.id 
        AND pp.price_date = CURRENT_DATE
    ) THEN p.id END) AS products_without_price,
    COUNT(DISTINCT CASE WHEN pp.status = 'PENDING' THEN p.id END) AS products_pending,
    COUNT(DISTINCT CASE WHEN pp.status = 'APPROVED' THEN p.id END) AS products_approved,
    COUNT(DISTINCT CASE WHEN pp.status = 'PUBLISHED' THEN p.id END) AS products_published,
    COUNT(DISTINCT CASE WHEN pp.status = 'REJECTED' THEN p.id END) AS products_rejected
FROM 
    users u
JOIN 
    user_roles ur ON u.id = ur.user_id
JOIN 
    roles r ON ur.role_id = r.id AND r.name = ?
JOIN 
    user_category_assignments uca ON u.id = uca.user_id
JOIN 
    categories c ON uca.category_id = c.id
JOIN (
    -- Para categorias de nível 1, encontre todas as categorias de nível 3 relacionadas
    SELECT c3.id, c1.id AS root_id
    FROM categories c1
    JOIN categories c2 ON c2.parent_id = c1.id AND c2.level = 2
    JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
    WHERE c1.level = 1
    
    UNION ALL
    
    -- Para categorias de nível 2, encontre todas as categorias de nível 3 relacionadas
    SELECT c3.id, c2.id AS root_id
    FROM categories c2
    JOIN categories c3 ON c3.parent_id = c2.id AND c3.level = 3
    WHERE c2.level = 2
    
    UNION ALL
    
    -- Categorias de nível 3 estão diretamente relacionadas a si mesmas
    SELECT c3.id, c3.id AS root_id
    FROM categories c3
    WHERE c3.level = 3
) AS category_mapping ON (
    c.id = category_mapping.root_id
)
JOIN 
    products p ON p.category_id = category_mapping.id
LEFT JOIN 
    product_prices pp ON p.id = pp.product_id 
    AND pp.price_date = CURRENT_DATE
WHERE 
    u.username = ? -- Substitua pelo nome de usuário desejado
GROUP BY 
    u.username, c.id, c.name, c.level
ORDER BY 
    c.level, c.name;"#;

        let stats = sqlx::query_as::<_, PriceStatusDTO>(sql)
            .bind(role_name)
            .bind(username)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to fetch product price status stats"))?;
        Ok(stats)
    }

    // 客户获取产品列表
    async fn get_product_list(
        &self,
        query: &ProductListQueryParams,
        tenant_hash: &str,
    ) -> Result<Vec<ProductListDTO>, AppError> {
        debug!("get_product_list: {:?}", query);
        let page_size = query.page_size.unwrap_or(12);
        let offset = (query.page.unwrap_or(1) - 1) * page_size;

        let tenant_id =
            sqlx::query_scalar!("SELECT id FROM tenants WHERE name_hash = ?", tenant_hash)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get tenant id"))?;

        if tenant_id == 0 {
            return Err(AppError::NotFound("Tenant not found".to_string()));
        }

        let sql = r#"SELECT 
        p.product_code, 
        p.name, 
        p.min_order_quantity,
        c1.name as category,
        c1.id as category_level1_id, 
        COALESCE(today_price.avg_price, history_price.avg_price, 0) as avg_price, 
        COALESCE(today_price.min_price, history_price.min_price, 0) as min_price,
        COALESCE(today_price.max_price, history_price.max_price, 0) as max_price,
        p.unit, 
        img.temp_url as image,
        p.is_disabled,
        COALESCE(ccd.discount_rate, 1) as discount_rate
    FROM 
        products p 
        JOIN categories c3 ON p.category_id = c3.id AND c3.level = 3
        JOIN categories c2 ON c3.parent_id = c2.id AND c2.level = 2
        JOIN categories c1 ON c2.parent_id = c1.id AND c1.level = 1
        -- 优先获取当日价格
        LEFT JOIN (
            SELECT * FROM product_prices 
            WHERE price_date = CURDATE() 
            AND status = 'PUBLISHED'
        ) today_price ON p.id = today_price.product_id
        -- 如果当日没有价格，则获取最近的历史价格
        LEFT JOIN (
            SELECT pp.* 
            FROM product_prices pp
            JOIN (
                SELECT product_id, MAX(price_date) as latest_date
                FROM product_prices
                WHERE status = 'PUBLISHED' 
                AND price_date < CURDATE()
                GROUP BY product_id
            ) latest ON pp.product_id = latest.product_id 
                      AND pp.price_date = latest.latest_date
                      AND pp.status = 'PUBLISHED'
        ) history_price ON p.id = history_price.product_id AND today_price.id IS NULL
        LEFT JOIN temp_image_urls img ON img.product_code = p.product_code
        LEFT JOIN customer_category_discounts ccd ON c1.id = ccd.category_id 
            AND ccd.status = true
            AND CURRENT_DATE BETWEEN ccd.start_date AND COALESCE(ccd.end_date, '9999-12-31')"#;

        let mut builder: QueryBuilder<MySql> = QueryBuilder::new(sql);
        builder.push(" AND ccd.tenant_id = ").push_bind(tenant_id);

        builder.push(" WHERE 1=1 AND (today_price.id IS NOT NULL OR history_price.id IS NOT NULL) AND p.is_disabled = 0 ");
        if let Some(category_id) = query.category_id {
            builder.push(" AND c2.id = ").push_bind(category_id);
        }
        if let Some(name) = &query.name {
            builder
                .push(" AND p.name LIKE ")
                .push_bind(format!("%{}%", name));
        }
        if let Some(is_disabled) = query.is_disabled {
            builder.push(" AND p.is_disabled = ").push_bind(is_disabled);
        }

        builder.push(" ORDER BY c1.sort_order DESC , p.id DESC");
        builder.push(" LIMIT ").push_bind(page_size);
        builder.push(" OFFSET ").push_bind(offset);

        let products = builder
            .build_query_as::<ProductListDTO>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get product list"))?;
        Ok(products)
    }

    async fn get_product_detail(
        &self,
        product_code: &str,
        is_owner: bool,
    ) -> Result<Option<ProductDetailResponse>, AppError> {
        // 根据是否是owner，获取不同的产品信息，owner获取该信息主要用于产品管理，非owner获取该信息主要用于客户获取产品信息
        let sql = if is_owner {
            r#"SELECT p.product_code, p.name, p.unit, p.product_description, p.brand, p.shelf_life,
         p.storage_conditions, p.pricing_method, p.tips, p.special_notes, p.min_order_quantity, p.tax_rate, img.temp_url as image FROM products p 
         JOIN temp_image_urls img ON p.product_code = img.product_code
         WHERE p.product_code = ?"#
        } else {
            r#"SELECT p.product_code, p.name, p.unit, p.product_description, p.brand, p.shelf_life,
         p.storage_conditions, p.pricing_method, p.tips, p.special_notes, p.min_order_quantity, p.tax_rate, img.temp_url as image FROM products p 
         INNER JOIN product_prices pp ON p.id = pp.product_id 
         INNER JOIN temp_image_urls img ON p.product_code = img.product_code
         WHERE p.is_disabled = 0 AND pp.price_date = (CURDATE()) AND p.product_code = ?"#
        };

        let basic = sqlx::query_as::<_, ProductDetailDTO>(sql)
            .bind(product_code)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get product detail"))?;

        // 2. 如果产品存在，再获取加工费用
        if let Some(product_detail) = basic {
            let processing_fee_sql = r#"SELECT
                    ppf.id AS processing_fee_id,
                    ppf.processing_type,
                    ppf.fee_type,
                    ppf.fee_value,
                    ppf.description,
                    ppf.is_checkbox,
                    ppfr.is_default
                FROM 
                    products p
                JOIN 
                    product_processing_fee_relations ppfr ON p.id = ppfr.product_id
                JOIN 
                    product_processing_fees ppf ON ppfr.processing_fee_id = ppf.id
                WHERE 
                    p.product_code = ?
                ORDER BY 
                    ppfr.sort_order, ppf.processing_type"#;

            let processing_fees = sqlx::query_as::<_, ProcessingFeeDTO>(processing_fee_sql)
                .bind(product_code)
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get product processing fees"))?;

            // 3. 将加工费用设置到产品详情中
            let product = ProductDetailResponse {
                product: ProductDetailDTO {
                    product_code: product_detail.product_code,
                    name: product_detail.name,
                    unit: product_detail.unit,
                    product_description: product_detail.product_description,
                    brand: product_detail.brand,
                    storage_conditions: product_detail.storage_conditions,
                    shelf_life: product_detail.shelf_life,
                    pricing_method: product_detail.pricing_method,
                    min_order_quantity: product_detail.min_order_quantity,
                    special_notes: product_detail.special_notes,
                    tips: product_detail.tips,
                    tax_rate: product_detail.tax_rate,
                    image: product_detail.image,
                },
                processing_fees: Some(processing_fees),
            };
            Ok(Some(product))
        } else {
            Ok(None)
        }
    }

    async fn deactivate_product(&self, product_code: &str) -> Result<(), AppError> {
        let sql = "UPDATE products SET is_disabled = NOT is_disabled WHERE product_code = ?";
        sqlx::query(sql)
            .bind(product_code)
            .execute(&self.pool)
            .await
            .map_err(map_db_err!("Failed to update product status"))?;
        Ok(())
    }

    async fn update_product(
        &self,
        product_code: &str,
        product: &UpdateProductRequestDTO,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        // 更新产品基本信息
        let sql = "UPDATE products SET name = ?, unit = ?, product_description = ?, brand = ?, shelf_life = ?, storage_conditions = ?, pricing_method = ?, tips = ?, special_notes = ?, min_order_quantity = ?, tax_rate = ? WHERE product_code = ?";
        sqlx::query(sql)
            .bind(&product.name)
            .bind(&product.unit)
            .bind(&product.product_description)
            .bind(&product.brand)
            .bind(&product.shelf_life)
            .bind(&product.storage_conditions)
            .bind(&product.pricing_method)
            .bind(&product.tips)
            .bind(&product.special_notes)
            .bind(product.min_order_quantity)
            .bind(product.tax_rate)
            .bind(product_code)
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to update product"))?;

        // 如果存在加工服务，先删除原有关联，再重新插入
        if let Some(processing_services) = &product.processing_services {
            // 获取产品ID
            let product_id = sqlx::query_scalar!(
                "SELECT id FROM products WHERE product_code = ?",
                product_code
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to get product id"))?;

            // 删除原有的所有关联
            sqlx::query!(
                "DELETE FROM product_processing_fee_relations WHERE product_id = ?",
                product_id
            )
            .execute(&mut *tx)
            .await
            .map_err(map_db_err!("Failed to delete product processing relations"))?;

            // 先对加工服务ID进行排序
            let mut sorted_services = processing_services.clone();
            sorted_services.sort();

            // 插入新的关联
            for (index, service_id) in sorted_services.iter().enumerate() {
                sqlx::query!(
                    "INSERT INTO product_processing_fee_relations (product_id, processing_fee_id, sort_order) VALUES (?, ?, ?)",
                    product_id,
                    service_id,
                    index as i32
                )
                .execute(&mut *tx)
                .await
                .map_err(map_db_err!("Failed to insert product processing relation"))?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_processing_fees(&self) -> Result<Vec<ProcessingFeeDTO>, AppError> {
        let sql = "SELECT id as processing_fee_id, processing_type, fee_type, fee_value, description, is_checkbox FROM product_processing_fees";
        let fees = sqlx::query_as::<_, ProcessingFeeDTO>(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get processing fees"))?;
        Ok(fees)
    }
}
