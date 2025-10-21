use market_saas_backend::{
    common::AppError,
    dto::order::{
        CreateOrderDTO, CreateOrderItem, DeliveryInfo, OrderQueryParams, OrderStatus,
        ProcessingService,
    },
    repositories::{my_sql_repository::MySqlRepository, order_traits::OrderRepository, TenantType},
};
use rust_decimal::Decimal;
use sqlx::{MySql, Pool};
use std::sync::Arc;

/// 集成测试辅助结构
struct TestContext {
    repo: MySqlRepository,
    pool: Arc<Pool<MySql>>,
}

impl TestContext {
    async fn new() -> Self {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:Nihaoccj123@localhost:3307/test_db".to_string());

        let pool = Arc::new(
            sqlx::MySqlPool::connect(&database_url)
                .await
                .expect("Failed to connect to test database"),
        );

        let repo = MySqlRepository::new((*pool).clone());

        Self { repo, pool }
    }

    /// 清理测试数据（按照外键依赖顺序）
    async fn cleanup(&self) {
        let cleanup_sqls = vec![
            "DELETE FROM reconciliation_statement_details",
            "DELETE FROM reconciliation_statement_orders",
            "DELETE FROM reconciliation_statements",
            "DELETE FROM refund_records",
            "DELETE FROM exchange_items",
            "DELETE FROM return_exchange_records",
            "DELETE FROM order_status_history",
            "DELETE FROM order_details",
            "DELETE FROM provider_orders_assignments",
            "DELETE FROM orders",
            "DELETE FROM delivery_staff",
            "DELETE FROM customer_category_discounts",
            "DELETE FROM customer_category_discount_history",
            "DELETE FROM user_category_assignments",
            "DELETE FROM product_processing_fee_relations",
            "DELETE FROM product_prices",
            "DELETE FROM products",
            "DELETE FROM categories",
            "DELETE FROM user_roles",
            "DELETE FROM role_permissions",
            "DELETE FROM users",
            "DELETE FROM tenant_relationships",
            "DELETE FROM provider_financial_profiles",
            "DELETE FROM messages",
            "DELETE FROM temp_image_urls",
            "DELETE FROM tenants",
            "DELETE FROM roles",
            "DELETE FROM permissions",
        ];

        for sql in cleanup_sqls {
            let _ = sqlx::query(sql).execute(&*self.pool).await;
        }
    }

    /// 创建测试租户 - 使用8字符以内的hash
    async fn setup_test_tenant(&self, tenant_hash: &str, tenant_type: &str) -> i32 {
        // 确保hash不超过8个字符
        let short_hash = if tenant_hash.len() > 8 {
            &tenant_hash[..8]
        } else {
            tenant_hash
        };

        let tenant_name = format!("测试{}", tenant_type);

        let result = sqlx::query!(
            r#"INSERT INTO tenants (name, tenant_type, name_hash, address, status)
               VALUES (?, ?, ?, '测试地址', 'ACTIVE')
               ON DUPLICATE KEY UPDATE name = VALUES(name)"#,
            tenant_name,
            tenant_type,
            short_hash
        )
        .execute(&*self.pool)
        .await;

        match result {
            Ok(res) => {
                // 如果是更新操作，需要查询现有ID
                if res.last_insert_id() == 0 {
                    let existing = sqlx::query!(
                        "SELECT id FROM tenants WHERE name_hash = ?",
                        short_hash
                    )
                    .fetch_one(&*self.pool)
                    .await
                    .expect("Failed to get existing tenant");
                    existing.id
                } else {
                    res.last_insert_id() as i32
                }
            }
            Err(e) => panic!("Failed to create test tenant: {:?}", e),
        }
    }

    /// 创建测试分类
    async fn setup_test_category(&self, category_name: &str) -> i32 {
        let result = sqlx::query!(
            r#"INSERT INTO categories (name, level, parent_id, sort_order)
               VALUES (?, 1, NULL, 0)
               ON DUPLICATE KEY UPDATE name = VALUES(name)"#,
            category_name
        )
        .execute(&*self.pool)
        .await
        .expect("Failed to create test category");

        if result.last_insert_id() == 0 {
            let existing = sqlx::query!(
                "SELECT id FROM categories WHERE name = ?",
                category_name
            )
            .fetch_one(&*self.pool)
            .await
            .expect("Failed to get existing category");
            existing.id
        } else {
            result.last_insert_id() as i32
        }
    }

    /// 创建测试产品
    async fn setup_test_product(&self, category_id: i32, product_code: &str, name: &str) -> i32 {
        let result = sqlx::query!(
            "INSERT INTO products (category_id, product_code, name, unit, is_disabled) VALUES (?, ?, ?, '斤', false)",
            category_id,
            product_code,
            name
        )
        .execute(&*self.pool)
        .await
        .expect("Failed to create test product");

        result.last_insert_id() as i32
    }

    /// 创建测试配送员
    async fn setup_test_delivery_staff(&self, provider_id: i32, name: &str, id_card: &str) -> i32 {
        let result = sqlx::query!(
            "INSERT INTO delivery_staff (provider_id, name, phone, id_card, created_by) VALUES (?, ?, '13800000000', ?, 'test')",
            provider_id,
            name,
            id_card
        )
        .execute(&*self.pool)
        .await
        .expect("Failed to create test delivery staff");

        result.last_insert_id() as i32
    }

    /// 创建简单的测试订单DTO
    fn create_simple_test_order_dto(&self, category_id: i32) -> CreateOrderDTO {
        CreateOrderDTO {
            items: vec![CreateOrderItem {
                product_code: "TEST001".to_string(),
                product_name: "测试牛肉".to_string(),
                category_id,
                category_name: "测试分类".to_string(),
                unit: "斤".to_string(),
                quantity: Decimal::new(500, 2), // 5.00
                original_price: Decimal::new(3000, 2), // 30.00
                discount_rate: Decimal::new(10, 2), // 0.10 (10% 折扣)
                price: Decimal::new(2700, 2), // 27.00 折后价
                total: Decimal::new(13500, 2), // 135.00 总价
                original_amount: Decimal::new(15000, 2), // 150.00 原价总额
                processing_services: vec![ProcessingService {
                    name: "切片".to_string(),
                    description: Some("切成薄片".to_string()),
                }],
                remark: Some("新鲜".to_string()),
            }],
            total_amount: Decimal::new(13500, 2), // 135.00
            delivery_info: DeliveryInfo {
                delivery_date: "2024-12-31".to_string(),
                delivery_address: "北京市朝阳区测试地址123号".to_string(),
                contact_name: "张三".to_string(),
                contact_phone: "13800138000".to_string(),
            },
        }
    }

    /// 清理测试数据
    async fn cleanup_test_data(&self) {
        let cleanup_queries = vec![
            "DELETE FROM order_details WHERE product_code LIKE 'TEST%'",
            "DELETE FROM orders WHERE order_code LIKE 'ODR-%'",
            "DELETE FROM tenants WHERE name_hash IN ('TESTCUST', 'TESTPROV', 'TESTMRKT')",
            "DELETE FROM categories WHERE name LIKE '测试%'",
        ];

        for query in cleanup_queries {
            let _ = sqlx::query(query).execute(&*self.pool).await;
        }
    }
}

/// 创建订单集成测试
#[tokio::test]
async fn test_create_order_integration() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    // 设置测试数据 - 使用8字符hash
    let market_id = ctx.setup_test_tenant("TESTMRKT", "MARKET").await;
    let customer_hash = "TESTCUST"; // 8字符
    let customer_id = ctx.setup_test_tenant(customer_hash, "CUSTOMER").await;
    let category_id = ctx.setup_test_category("测试肉类").await;

    println!("Market ID: {}, Customer ID: {}, Category ID: {}", market_id, customer_id, category_id);

    // 创建测试订单
    let order_dto = ctx.create_simple_test_order_dto(category_id);

    // 执行创建订单
    let result = ctx.repo.create_order(market_id, customer_hash, &order_dto).await;

    // 验证结果
    assert!(result.is_ok(), "创建订单失败: {:?}", result.err());
    let order_response = result.unwrap();

    assert!(!order_response.order_code.is_empty());
    assert!(order_response.order_code.starts_with("ODR-"));
    assert_eq!(order_response.order_status, "PENDING");
    assert_eq!(order_response.actual_amount, order_dto.total_amount);

    println!("创建的订单代码: {}", order_response.order_code);

    // 清理测试数据
    ctx.cleanup_test_data().await;
}

/// 测试查询订单的分页和过滤功能
#[tokio::test]
async fn test_get_orders_by_tenant_integration() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    // 设置测试数据
    let market_id = ctx.setup_test_tenant("TESTMRKT", "MARKET").await;
    let customer_hash = "TESTCUST";
    let customer_id = ctx.setup_test_tenant(customer_hash, "CUSTOMER").await;
    let category_id = ctx.setup_test_category("测试肉类").await;

    // 创建2个测试订单
    for i in 1..=2 {
        let mut order_dto = ctx.create_simple_test_order_dto(category_id);
        order_dto.delivery_info.contact_name = format!("测试客户{}", i);
        
        let result = ctx.repo.create_order(market_id, customer_hash, &order_dto).await;
        assert!(result.is_ok(), "创建订单{}失败: {:?}", i, result.err());
    }

    // 测试获取订单列表
    let query_params = OrderQueryParams {
        page: Some(1),
        page_size: Some(10),
        order_status: Some("PENDING".to_string()),
    };

    let result = ctx
        .repo
        .get_orders_by_tenant(customer_hash, "CUSTOMER", &query_params)
        .await;

    assert!(result.is_ok(), "获取订单列表失败: {:?}", result.err());
    let orders = result.unwrap();
    assert_eq!(orders.len(), 2, "应该返回2个订单");

    for order in &orders {
        assert_eq!(order.order_status, "PENDING");
        assert!(order.order_code.starts_with("ODR-"));
        println!("订单: {}, 状态: {}", order.order_code, order.order_status);
    }

    // 清理测试数据
    ctx.cleanup_test_data().await;
}

/// 测试根据订单代码获取订单详情
#[tokio::test]
async fn test_get_order_by_code_integration() {
    let ctx = TestContext::new().await;
    ctx.cleanup_test_data().await;

    // 设置测试数据
    let market_id = ctx.setup_test_tenant("TESTMRKT", "MARKET").await;
    let customer_hash = "TESTCUST";
    let customer_id = ctx.setup_test_tenant(customer_hash, "CUSTOMER").await;
    let category_id = ctx.setup_test_category("测试肉类").await;

    // 创建测试订单
    let order_dto = ctx.create_simple_test_order_dto(category_id);
    let create_result = ctx.repo.create_order(market_id, customer_hash, &order_dto).await;
    assert!(create_result.is_ok());
    let order_response = create_result.unwrap();

    // 测试根据订单代码获取订单详情
    let result = ctx
        .repo
        .get_order_by_order_code(&order_response.order_code)
        .await;

    assert!(result.is_ok(), "获取订单详情失败: {:?}", result.err());
    let order_detail = result.unwrap();
    
    assert!(order_detail.is_some(), "订单详情不应该为空");
    let order_detail = order_detail.unwrap();
    
    assert_eq!(order_detail.order.order_code, order_response.order_code);
    assert_eq!(order_detail.items.len(), 1);
    assert_eq!(order_detail.items[0].product_code, "TEST001");

    println!("订单详情验证成功: {}", order_detail.order.order_code);

    // 清理测试数据
    ctx.cleanup_test_data().await;
}

#[cfg(test)]
mod setup {
    use super::*;

    /// 测试环境初始化
    pub async fn init_test_database() {
        let ctx = TestContext::new().await;
        
        // 创建基础测试数据
        let _market_id = ctx.setup_test_tenant("TESTMRKT", "MARKET").await;
        let _customer_id = ctx.setup_test_tenant("TESTCUST", "CUSTOMER").await;
        let _provider_id = ctx.setup_test_tenant("TESTPROV", "PROVIDER").await;
        let _category_id = ctx.setup_test_category("测试分类").await;
        
        println!("测试数据库初始化完成");
    }

    /// 测试数据清理
    pub async fn cleanup_test_database() {
        let ctx = TestContext::new().await;
        ctx.cleanup_test_data().await;
        println!("测试数据库清理完成");
    }
}
