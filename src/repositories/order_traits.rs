use crate::{
    common::AppError,
    dto::order::{
        AcceptedOrderResponseDTO, ProductsSummaryWithOrdersDTO, ProviderDashboardStatsDTO,
    },
    map_db_err,
    models::{
        tenant_type::TenantType,
    },
};

use super::my_sql_repository::MySqlRepository;
use async_trait::async_trait;

use tracing::info;

#[async_trait]
pub(crate) trait OrderRepository: Send + Sync {
    async fn get_after_sale_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &TenantType,
    ) -> Result<Vec<AcceptedOrderResponseDTO>, AppError>;

    async fn fetch_preparation_summary(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<ProductsSummaryWithOrdersDTO>, AppError>;

    async fn get_provider_dashboard_stats(
        &self,
        provider_hash: &str,
    ) -> Result<ProviderDashboardStatsDTO, AppError>;
}

#[async_trait]
impl OrderRepository for MySqlRepository {
    
    async fn get_after_sale_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &TenantType,
    ) -> Result<Vec<AcceptedOrderResponseDTO>, AppError> {
        info!("Get accepted orders by tenant_hash: {}", tenant_hash);

        let orders = match tenant_type {
            TenantType::Provider => sqlx::query_as!(
                AcceptedOrderResponseDTO,
                r#"WITH provider_orders AS (
                        SELECT DISTINCT poa.order_id
                        FROM provider_orders_assignments poa
                        JOIN tenants t ON poa.provider_id = t.id
                        WHERE t.name_hash = ?
                        AND t.tenant_type = ?
                        AND t.deleted_at IS NULL
                    )
                    SELECT 
                        o.order_code,
                        o.order_status, 
                        COALESCE(MIN(osh.created_at), o.created_at) as accepted_at
                    FROM orders o
                    JOIN provider_orders po ON o.id = po.order_id
                    LEFT JOIN order_status_history osh ON o.id = osh.order_id
                    WHERE (o.order_status = 'AFTER_SALE' OR o.order_status = 'ACCEPTED')
                    AND (osh.to_status = 'AFTER_SALE' OR osh.to_status = 'ACCEPTED')
                    AND o.deleted_at IS NULL
                    GROUP BY o.order_code, o.order_status, o.created_at
                    ORDER BY accepted_at DESC"#,
                tenant_hash,
                tenant_type.to_string()
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get accepted orders"))?,
            TenantType::Customer => sqlx::query_as!(
                AcceptedOrderResponseDTO,
                r#"WITH customer_orders AS (
                        SELECT o.id as order_id
                        FROM orders o
                        JOIN tenants t ON o.customer_id = t.id
                        WHERE t.name_hash = ?
                        AND t.tenant_type = ?
                        AND t.deleted_at IS NULL
                    )
                    SELECT 
                        o.order_code,
                        o.order_status, 
                        COALESCE(MIN(osh.created_at), o.created_at) as accepted_at
                    FROM orders o
                    JOIN customer_orders co ON o.id = co.order_id
                    LEFT JOIN order_status_history osh ON o.id = osh.order_id
                    WHERE (o.order_status = 'AFTER_SALE' OR o.order_status = 'ACCEPTED')
                    AND (osh.to_status = 'AFTER_SALE' OR osh.to_status = 'ACCEPTED')
                    AND o.deleted_at IS NULL
                    GROUP BY o.order_code, o.order_status, o.created_at
                    ORDER BY accepted_at DESC"#,
                tenant_hash,
                tenant_type.to_string()
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get accepted orders"))?,
            TenantType::Market => sqlx::query_as!(
                AcceptedOrderResponseDTO,
                r#"SELECT 
                        o.order_code,
                        o.order_status, 
                        COALESCE(MIN(osh.created_at), o.created_at) as accepted_at
                    FROM orders o
                    JOIN tenants t ON o.market_id = t.id
                    LEFT JOIN order_status_history osh ON o.id = osh.order_id
                    WHERE t.name_hash = ?
                    AND t.tenant_type = ?
                    AND (o.order_status = 'AFTER_SALE' OR o.order_status = 'ACCEPTED')
                    AND (osh.to_status = 'AFTER_SALE' OR osh.to_status = 'ACCEPTED')
                    AND o.deleted_at IS NULL
                    AND t.deleted_at IS NULL
                    GROUP BY o.order_code, o.order_status, o.created_at
                    ORDER BY accepted_at DESC"#,
                tenant_hash,
                tenant_type.to_string()
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get accepted orders"))?,
        };

        Ok(orders)
    }

    async fn fetch_preparation_summary(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<ProductsSummaryWithOrdersDTO>, AppError> {
        use tracing::debug;
        debug!("Fetching preparation summary for provider_hash: {}", provider_hash);
        
        let orders = sqlx::query_as!(
            ProductsSummaryWithOrdersDTO,
            r#"SELECT
                od.product_code,
                od.product_name,
                SUM(od.ordered_qty) AS total_quantity,
                od.unit,
                od.processing_requirements,
                c.name AS customer_name,
                od.remark
            FROM
                provider_orders_assignments poa
            JOIN
                tenants t ON poa.provider_id = t.id
            JOIN
                orders o ON poa.order_id = o.id
            JOIN
                order_details od ON o.id = od.order_id
            JOIN
                tenants c ON o.customer_id = c.id
            WHERE
                t.name_hash = ?
                AND t.tenant_type = 'PROVIDER'
                AND t.deleted_at IS NULL
                AND o.order_status IN ('ASSIGNED', 'SUPPLIER_PREPARING')
                AND o.deleted_at IS NULL
                AND o.created_at >= CURDATE()
            GROUP BY
                od.product_code, od.product_name, od.unit, od.processing_requirements, c.name, od.remark
            ORDER BY
                total_quantity DESC;"#,
                provider_hash
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            debug!("Error fetching preparation summary: {:?}", e);
            map_db_err!("Failed to get product summaries with orders")(e)
        })?;
        
        debug!("Fetched {} preparation summary records", orders.len());
        Ok(orders)
    }

    async fn get_provider_dashboard_stats(
        &self,
        provider_hash: &str,
    ) -> Result<ProviderDashboardStatsDTO, AppError> {
        use tracing::debug;
        debug!("Fetching dashboard stats for provider_hash: {}", provider_hash);

        // 查询待配送订单总数（ASSIGNED 或 SUPPLIER_PREPARING）
        let pending_delivery = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT o.id) as count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            WHERE t.name_hash = ?
                AND t.tenant_type = 'PROVIDER'
                AND t.deleted_at IS NULL
                AND o.order_status IN ('ASSIGNED', 'SUPPLIER_PREPARING')
                AND o.deleted_at IS NULL
            "#,
            provider_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get pending delivery orders count"))?
        .count;

        // 查询配送中订单总数（SUPPLIER_DELIVERING）
        let delivering = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT o.id) as count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            WHERE t.name_hash = ?
                AND t.tenant_type = 'PROVIDER'
                AND t.deleted_at IS NULL
                AND o.order_status = 'SUPPLIER_DELIVERING' OR o.order_status = 'EXCHANGE_DELIVERING'
                AND o.deleted_at IS NULL
            "#,
            provider_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get delivering orders count"))?
        .count;

        // 查询已完成配送订单总数（COMPLETED）
        let completed = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT o.id) as count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            WHERE t.name_hash = ?
                AND t.tenant_type = 'PROVIDER'
                AND t.deleted_at IS NULL
                AND o.order_status = 'COMPLETED'
                AND o.deleted_at IS NULL
            "#,
            provider_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get completed orders count"))?
        .count;

        // 查询退换货任务总数（return_exchange_records 表中 status = 'PENDING'）
        let return_exchange_tasks = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT rer.id) as count
            FROM return_exchange_records rer
            JOIN order_details od ON rer.order_detail_id = od.id
            JOIN orders o ON od.order_id = o.id
            JOIN provider_orders_assignments poa ON o.id = poa.order_id
            JOIN tenants t ON poa.provider_id = t.id
            WHERE t.name_hash = ?
                AND t.tenant_type = 'PROVIDER'
                AND t.deleted_at IS NULL
                AND rer.status = 'PENDING'
                AND o.deleted_at IS NULL
            "#,
            provider_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get return exchange tasks count"))?
        .count;

        // 查询待备货 SKU 总数（订单状态为 ASSIGNED 的订单的 product_code 去重）
        let pending_stock_skus = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT od.product_code) as count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            JOIN order_details od ON o.id = od.order_id
            WHERE t.name_hash = ?
                AND t.tenant_type = 'PROVIDER'
                AND t.deleted_at IS NULL
                AND o.order_status = 'ASSIGNED'
                AND o.deleted_at IS NULL
            "#,
            provider_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get pending stock SKUs count"))?
        .count;

        debug!(
            "Dashboard stats - pending_delivery: {}, delivering: {}, completed: {}, return_exchange: {}, pending_stock: {}",
            pending_delivery, delivering, completed, return_exchange_tasks, pending_stock_skus
        );

        Ok(ProviderDashboardStatsDTO {
            pending_delivery_orders: pending_delivery as i64,
            delivering_orders: delivering as i64,
            completed_orders: completed as i64,
            return_exchange_tasks: return_exchange_tasks as i64,
            pending_stock_skus: pending_stock_skus as i64,
        })
    }
}