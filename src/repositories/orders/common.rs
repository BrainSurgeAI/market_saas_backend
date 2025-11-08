use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::order::{
        ExchangeAndReturnOrderDetailResponse, OrderDetail, OrderDetailResponse, OrderItem,
        OrderQueryParams, OrderReceipt, OrderResponse, ReceiptOperationType,
    },
    map_db_err,
    models::{claims::Claims, tenant_type::TenantType},
};
use async_trait::async_trait;
use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error};

#[async_trait]
pub(crate) trait CommonOrderRepository: Send + Sync {
    async fn get_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &str,
        query_params: &OrderQueryParams,
    ) -> Result<Vec<OrderResponse>, AppError>;

    async fn order_by_order_code(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<Option<OrderDetailResponse>, AppError>;

    async fn get_exchange_and_return_order_details_by_order_code(
        &self,
        order_code: &str,
    ) -> Result<Vec<ExchangeAndReturnOrderDetailResponse>, AppError>;
}

#[async_trait]
impl CommonOrderRepository for MySqlRepository {
    async fn get_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &str,
        query_params: &OrderQueryParams,
    ) -> Result<Vec<OrderResponse>, AppError> {
        let page = query_params.page.unwrap_or(1);
        let page_size = query_params.page_size.unwrap_or(10);
        let offset = (page - 1) * page_size;

        let record = sqlx::query!(
            r#"select id from tenants where name_hash=? and tenant_type=? and status='ACTIVE'"#,
            tenant_hash,
            tenant_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get tenant by id"))?
        .ok_or_else(|| {
            return AppError::NotFound(format!(
                "Can not find Tenant by type {} name_hash {} ",
                tenant_type, tenant_hash
            ));
        })?;

        let basic_query = r#"SELECT 
                   o.order_code, 
                   o.delivery_address, 
                   o.total_amount,
                   o.actual_amount, 
                   o.delivery_date, 
                   o.order_status,
                   o.created_at,
                   o.after_sale_at
                   FROM orders o {JOIN_CLAUSE} WHERE 1=1 "#;

        let mut builder: QueryBuilder<MySql>;
        if tenant_type == TenantType::Provider.to_string() {
            let query = &basic_query.replace(
                "{JOIN_CLAUSE}",
                "JOIN provider_orders_assignments po ON po.order_id=o.id",
            );

            builder = QueryBuilder::new(query);
            builder.push(" AND po.provider_id= ").push_bind(record.id);
        } else {
            let query = &basic_query.replace("{JOIN_CLAUSE}", "");

            builder = QueryBuilder::new(query);
            builder.push(" AND (o.market_id=").push_bind(record.id);
            builder.push(" OR o.customer_id=").push_bind(record.id);
            builder.push(") ");
        }

        if let Some(order_status) = query_params.order_status.as_ref() {
            debug!("order_status: {}", order_status);
            builder.push(" AND o.order_status=").push_bind(order_status);
        }

        builder.push(" ORDER BY o.created_at DESC ");
        builder.push(" LIMIT ").push_bind(page_size);
        builder.push(" OFFSET ").push_bind(offset);

        let orders = builder
            .build_query_as::<OrderResponse>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get customer orders"))?;
        Ok(orders)
    }

    /// Get order by order code and tenant hash
    ///
    /// # Arguments
    /// * `order_code` - Order code
    /// * `tenant_hash` - Tenant hash
    ///
    /// # Returns
    /// The order detail response
    async fn order_by_order_code(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<Option<OrderDetailResponse>, AppError> {
        let record = sqlx::query!(
            r#"select id from tenants where name_hash=? and tenant_type=? and status='ACTIVE'"#,
            claims.tenant_hash,
            claims.tenant_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get tenant by id"))?
        .ok_or_else(|| {
            return AppError::NotFound(format!(
                "Can not find Tenant by type {} name_hash {} ",
                claims.tenant_type, claims.tenant_hash
            ));
        })?;

        // 基础查询：customer_name 总是从 customer_id 对应的 tenant 获取
        // provider_name 根据租户类型不同，从不同的地方获取
        let basic_query = r#"
        SELECT o.id, o.order_code, tc.name as customer_name, o.order_status,
            o.total_amount, o.discount_amount, o.actual_amount, o.delivery_date, o.delivery_address,
            o.contact_name, o.contact_phone, o.remark, o.created_by, o.created_at, o.confirmed_by,
            o.confirmed_at, o.processed_by, o.processed_at, o.stocked_by, o.stocked_at, o.after_sale_at,
            o.rejected_by, o.rejected_at, o.reject_reason, o.completed_by, o.completed_at,
            ds.name as delivery_staff_name, ds.phone as delivery_staff_phone, {PROVIDER_NAME_SELECT}
        FROM orders o 
        INNER JOIN tenants tc ON o.customer_id = tc.id 
        {JOIN_CLAUSE}
        LEFT JOIN delivery_staff ds ON o.delivery_staff_id = ds.id 
        WHERE 1=1 "#;

        let mut builder: QueryBuilder<MySql>;
        match TenantType::try_from(claims.tenant_type.as_str())? {
            TenantType::Provider => {
                // Provider: provider_name 从 provider_orders_assignments 关联的 provider 获取
                let query = basic_query
                    .replace("{PROVIDER_NAME_SELECT}", "tp.name as provider_name")
                    .replace(
                        "{JOIN_CLAUSE}",
                        r#" INNER JOIN provider_orders_assignments po ON po.order_id=o.id 
                            INNER JOIN tenants tp ON po.provider_id = tp.id "#,
                    );
                builder = QueryBuilder::new(&query);
                builder.push(" AND tp.id= ").push_bind(record.id);
                builder.push(" AND tp.name_hash= ").push_bind(claims.tenant_hash.as_str());
            }
            TenantType::Customer => {
                // Customer: provider_name 从 market_id 对应的 market tenant 获取
                let query = basic_query
                    .replace("{PROVIDER_NAME_SELECT}", "tm.name as provider_name")
                    .replace(
                        "{JOIN_CLAUSE}",
                        r#" INNER JOIN tenants tm ON o.market_id = tm.id "#,
                    );
                builder = QueryBuilder::new(&query);
                builder.push(" AND tc.id= ").push_bind(record.id);
                builder.push(" AND tc.name_hash= ").push_bind(claims.tenant_hash.as_str());
            }
            TenantType::Market => {
                // Market: provider_name 从 provider_orders_assignments 关联的 provider 获取
                // 如果订单没有分配 provider，则 provider_name 为空（使用 LEFT JOIN）
                let query = basic_query
                    .replace("{PROVIDER_NAME_SELECT}", "tp.name as provider_name")
                    .replace(
                        "{JOIN_CLAUSE}",
                        r#" INNER JOIN tenants tm ON o.market_id = tm.id 
                            LEFT JOIN provider_orders_assignments po ON po.order_id = o.id 
                            LEFT JOIN tenants tp ON po.provider_id = tp.id "#,
                    );
                builder = QueryBuilder::new(&query);
                builder.push(" AND tm.id= ").push_bind(record.id);
                builder.push(" AND tm.name_hash= ").push_bind(claims.tenant_hash.as_str());
            }
        };

        builder.push(" AND o.order_code= ").push_bind(order_code);

        let order = builder
            .build_query_as::<OrderItem>()
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get order by code"))?;

        let order = match order {
            Some(o) => o,
            None => return Ok(None),
        };

        let items = sqlx::query_as!(
            OrderDetail,
            r#"
        SELECT 
            od.id,
            od.product_code,
            od.product_name,
            od.category_id,
            od.category_name,
            od.unit,
            od.quantity,
            od.original_price,
            od.discount_rate,
            od.actual_price,
            od.actual_amount,
            od.total_amount,
            od.accepted_quantity,
            od.processing_requirements,
            od.remark,
            od.status,
            pdi.actual_qty AS delivered_quantity
        FROM order_details od
        LEFT JOIN provider_delivery_items pdi ON od.id = pdi.order_detail_id
        LEFT JOIN provider_deliveries pd ON pdi.delivery_id = pd.id
        WHERE od.order_id = ?
        "#,
            order.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order details"))?;

        let receipt_rows = sqlx::query!(
            r#"
        SELECT 
            re.order_detail_id,
            o.order_code,
            od.product_code,
            od.product_name,
            re.operation_type,
            re.quantity,
            re.reason,
            od.unit,
            re.evidence_images
        FROM return_exchange_records re
        JOIN order_details od ON re.order_detail_id = od.id
        JOIN orders o ON od.order_id = o.id
        WHERE o.order_code = ?
        ORDER BY re.created_at DESC
        "#,
            order_code
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get receipt records"))?;

        let receipts = receipt_rows
            .into_iter()
            .map(|r| {
                let operation_type = ReceiptOperationType::try_from(r.operation_type)
                    .map_err(|e| AppError::Validation(format!("无效的收据操作类型: {}", e)))?;

                Ok(OrderReceipt {
                    order_detail_id: r.order_detail_id,
                    order_code: r.order_code,
                    product_code: r.product_code,
                    product_name: r.product_name,
                    operation_type,
                    quantity: r.quantity,
                    reason: r.reason,
                    unit: r.unit,
                    evidence_images: r.evidence_images,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok(Some(OrderDetailResponse {
            order,
            items,
            receipts,
        }))
    }

    async fn get_exchange_and_return_order_details_by_order_code(
        &self,
        order_code: &str,
    ) -> Result<Vec<ExchangeAndReturnOrderDetailResponse>, AppError> {
        let order_details = sqlx::query_as!(
            ExchangeAndReturnOrderDetailResponse,
            r#"
                SELECT
                    rer.order_detail_id      AS "order_detail_id!",
                    rer.operation_type       AS "operation_type!",
                    rer.quantity             AS "quantity!",
                    rer.reason               AS "reason!",
                    p.unit                   AS "unit!",
                    rer.status               AS "status!",
                    p.name                   AS "product_name!",
                    rer.evidence_images      AS "evidence_images?",
                    rer.processed_by         AS "processed_by?",
                    rer.processed_at         AS "processed_at?",
                    rer.actual_quantity      AS "actual_quantity?"
                FROM return_exchange_records rer
                INNER JOIN order_details od ON od.id = rer.order_detail_id
                INNER JOIN orders o ON o.id = od.order_id
                INNER JOIN products p ON p.product_code = od.product_code
                WHERE o.order_code = ?
                ORDER BY rer.created_at DESC
                "#,
            order_code
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!(
            "Failed to get exchange and return order details"
        ))?;
        Ok(order_details)
    }
}
