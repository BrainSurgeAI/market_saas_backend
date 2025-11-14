use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::order::{
        ExchangeAndReturnOrderDetailResponse, OrderDetail, OrderDetailResponse,
        OrderQueryParams, OrderResponse, ReceiptItem, ReceiptOperationType, ReceiptResponse,
    },
    map_db_err,
    models::{claims::Claims, tenant_type::TenantType},
};
use async_trait::async_trait;
use sqlx::{MySql, QueryBuilder};
use tracing::{debug, error};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct OrderBaseInfoResponse {
    pub(crate) id: i32,

    #[serde(rename = "customerName")]
    pub(crate) customer_name: String,

    #[serde(rename = "orderedAmount")]
    pub(crate) ordered_amount: Decimal,

    #[serde(rename = "discountAmount")]
    pub(crate) discount_amount: Decimal,

    #[serde(rename = "netAmount")]
    pub(crate) net_amount: Decimal,

    #[serde(rename = "deliveryAddress")]
    pub(crate) delivery_address: String,

    #[serde(rename = "deliveryDate")]
    pub(crate) delivery_date: NaiveDate,

    #[serde(rename = "orderStatus")]
    pub(crate) order_status: String,

    #[serde(rename = "shipperName")]
    pub(crate) shipper_name: Option<String>,

    #[serde(rename = "shipperPhone")]
    pub(crate) shipper_phone: Option<String>,

    #[serde(rename = "receiverName")]
    pub(crate) receiver_name: Option<String>,

    #[serde(rename = "receiverPhone")]
    pub(crate) receiver_phone: Option<String>,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,
}

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

        let tenant_id = self
            .get_tenant_id_by_tenant_hash_and_tenant_type(tenant_hash, tenant_type)
            .await?;

        let basic_query = r#"SELECT 
                   o.order_code, 
                   o.delivery_address, 
                   o.ordered_amount,
                   o.net_amount, 
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
            builder.push(" AND po.provider_id= ").push_bind(tenant_id);
        } else {
            let query = &basic_query.replace("{JOIN_CLAUSE}", "");

            builder = QueryBuilder::new(query);
            builder.push(" AND (o.market_id=").push_bind(tenant_id);
            builder.push(" OR o.customer_id=").push_bind(tenant_id);
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
    /// 根据不同的租户类型返回不同的数据：
    /// - PROVIDER: 返回自己的发货记录（provider_delivery_items）
    /// - MARKET: 返回 PROVIDER 的发货记录和市场验收记录
    /// - CUSTOMER: 返回 MARKET 的签收数量（作为发货量）和客户验收记录
    ///
    /// # Arguments
    /// * `order_code` - Order code
    /// * `claims` - Claims containing tenant information
    ///
    /// # Returns
    /// The order detail response
    async fn order_by_order_code(
        &self,
        order_code: &str,
        claims: &Claims,
    ) -> Result<Option<OrderDetailResponse>, AppError> {
        let tenant_type = TenantType::try_from(claims.tenant_type.as_str())?;

        let tenant_id = self
            .get_tenant_id_by_tenant_hash_and_tenant_type(
                claims.tenant_hash.as_str(),
                claims.tenant_type.as_str(),
            )
            .await?;

        let (order_base, order_items) = match tenant_type {
            TenantType::Provider => {
                let order_base = sqlx::query_as!(
                    OrderBaseInfoResponse,
                    r#"
                    SELECT o.id, o.delivery_date, o.created_at, o.order_status, o.confirmed_by AS receiver_name, t.name AS customer_name,
                    o.ordered_amount, o.discount_amount, o.net_amount, o.market_contact_number AS receiver_phone,
                    t.address AS delivery_address, pd.delivered_by AS shipper_name, pd.delivery_contact_number AS shipper_phone
                    FROM orders o 
                    INNER JOIN provider_orders_assignments poa ON poa.order_id = o.id
                    LEFT JOIN provider_deliveries pd ON pd.assignment_id = poa.id
                    INNER JOIN tenants t ON o.market_id = t.id
                    WHERE o.order_code = ? AND poa.provider_id = ?
                    "#,
                    order_code,
                    tenant_id
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get order base info"))?
                .ok_or_else(|| AppError::NotFound(format!("Order not found")))?;

                let order_items = 
                    // PROVIDER: 查询自己的发货记录
                    // 只查询分配给该 PROVIDER 的订单，并且只统计该 PROVIDER 的发货记录
                    sqlx::query_as!(
                        OrderDetail,
                        r#"
                        SELECT 
                            od.id,
                            od.product_code,
                            od.product_name,
                            od.category_id,
                            od.category_name,
                            od.unit,
                            od.ordered_qty,
                            od.unit_price,
                            od.discount_rate,
                            od.discounted_unit_price,
                            od.net_amount,
                            od.ordered_amount,
                            od.processing_requirements,
                            od.remark,
                            COALESCE(SUM(CASE WHEN pd.assignment_id = poa.id THEN pdi.actual_qty ELSE 0 END), 0) AS delivered_quantity,
                            COALESCE((
                                SELECT oii.inspected_qty
                                FROM order_inspection_items oii
                                INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
                                WHERE oi.order_id = od.order_id
                                  AND oii.order_detail_id = od.id
                                  AND oi.inspected_by_type = 'MARKET'
                                ORDER BY oi.inspection_round DESC
                                LIMIT 1
                            ), 0) AS market_inspected_quantity,
                            COALESCE((
                                SELECT oii.inspected_qty
                                FROM order_inspection_items oii
                                INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
                                WHERE oi.order_id = od.order_id
                                  AND oii.order_detail_id = od.id
                                  AND oi.inspected_by_type = 'CUSTOMER'
                                ORDER BY oi.inspection_round DESC
                                LIMIT 1
                            ), 0) AS customer_inspected_quantity
                    
                        FROM order_details od
                        INNER JOIN provider_orders_assignments poa 
                            ON od.order_id = poa.order_id AND poa.provider_id = ?
                        LEFT JOIN provider_deliveries pd 
                            ON pd.assignment_id = poa.id
                        LEFT JOIN provider_delivery_items pdi 
                            ON pdi.delivery_id = pd.id AND pdi.order_detail_id = od.id
                        WHERE od.order_id = ?
                        GROUP BY 
                            od.id, od.product_code, od.product_name, od.category_id, od.category_name,
                            od.unit, od.ordered_qty, od.unit_price, od.discount_rate, od.discounted_unit_price,
                            od.ordered_amount, od.net_amount, 
                            od.processing_requirements, od.remark
                        "#,
                        tenant_id,
                        order_base.id
                    )
                    .fetch_all(&self.pool)
                    .await
                    .map_err(map_db_err!("Failed to get order details for provider"))?;
                    

                (order_base, order_items)

                
            }
            TenantType::Market => {
                let order_base = sqlx::query_as!(
                    OrderBaseInfoResponse,
                    r#"
                    SELECT o.id, o.delivery_date, o.created_at, o.order_status, o.contact_name AS receiver_name, 
                    o.contact_phone AS receiver_phone, t.name AS customer_name,
                    o.ordered_amount, o.discount_amount, o.net_amount, o.delivery_address, 
                    o.confirmed_by AS shipper_name, o.market_contact_number AS shipper_phone
                    FROM orders o
                    INNER JOIN tenants t ON o.market_id = t.id
                    INNER JOIN tenants t_customer ON o.customer_id = t_customer.id
                    WHERE o.order_code = ? AND t.id = ? AND t.tenant_type = 'MARKET'
                    "#,
                    order_code,
                    tenant_id
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get order base info"))?.ok_or_else(|| AppError::NotFound(format!("Order not found")))?;

                let order_items = sqlx::query_as!(
                    OrderDetail,
                    r#"
                    SELECT 
                        od.id,
                        od.product_code,
                        od.product_name,
                        od.category_id,
                        od.category_name,
                        od.unit,
                        od.ordered_qty,
                        od.unit_price,
                        od.discount_rate,
                        od.discounted_unit_price,
                        od.net_amount,
                        od.ordered_amount,
                        od.processing_requirements,
                        od.remark,
                        COALESCE(SUM(pdi.actual_qty), 0) AS delivered_quantity,
                        COALESCE((
                            SELECT oii.inspected_qty
                            FROM order_inspection_items oii
                            INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
                            WHERE oi.order_id = od.order_id
                              AND oii.order_detail_id = od.id
                              AND oi.inspected_by_type = 'MARKET'
                            ORDER BY oi.inspection_round DESC
                            LIMIT 1
                        ), 0) AS market_inspected_quantity,
                        COALESCE((
                            SELECT oii.inspected_qty
                            FROM order_inspection_items oii
                            INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
                            WHERE oi.order_id = od.order_id
                              AND oii.order_detail_id = od.id
                              AND oi.inspected_by_type = 'CUSTOMER'
                            ORDER BY oi.inspection_round DESC
                            LIMIT 1
                        ), 0) AS customer_inspected_quantity
                    FROM order_details od
                    LEFT JOIN provider_delivery_items pdi ON od.id = pdi.order_detail_id
                    LEFT JOIN provider_deliveries pd ON pdi.delivery_id = pd.id
                    WHERE od.order_id = ?
                    GROUP BY od.id, od.product_code, od.product_name, od.category_id, od.category_name,
                             od.unit, od.ordered_qty, od.unit_price, od.discount_rate, od.discounted_unit_price,
                             od.ordered_amount, od.net_amount, od.processing_requirements,
                             od.remark
                    "#,
                    order_base.id
                )
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get order details for market"))?;

                (order_base, order_items)
            }
            TenantType::Customer => {
                let order_base = sqlx::query_as!(
                    OrderBaseInfoResponse,
                    r#"
                    SELECT o.id, o.delivery_date, o.created_at, o.order_status, o.contact_name AS receiver_name, 
                    o.contact_phone AS receiver_phone, t.name AS customer_name, o.ordered_amount, o.discount_amount, 
                    o.net_amount, o.delivery_address, o.confirmed_by AS shipper_name, o.market_contact_number AS shipper_phone
                    FROM orders o
                    INNER JOIN tenants t ON o.customer_id = t.id
                    WHERE o.order_code = ? AND t.id = ? AND t.tenant_type = 'CUSTOMER'
                    "#,
                    order_code,
                    tenant_id
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get order base info"))?.ok_or_else(|| AppError::NotFound(format!("Order not found")))?;

                let order_items = sqlx::query_as!(
                    OrderDetail,
                    r#"
                    SELECT 
                        od.id,
                        od.product_code,
                        od.product_name,
                        od.category_id,
                        od.category_name,
                        od.unit,
                        od.ordered_qty,
                        od.unit_price,
                        od.discount_rate,
                        od.discounted_unit_price,
                        od.net_amount,
                        od.ordered_amount,
                        od.processing_requirements,
                        od.remark,
                        -- CUSTOMER 看到的发货量是 MARKET 的签收数量
                        COALESCE(SUM(CASE WHEN oi_market.inspected_by_type = 'MARKET' THEN oii_market.inspected_qty ELSE 0 END), 0) AS delivered_quantity,
                        COALESCE((
                            SELECT oii.inspected_qty
                            FROM order_inspection_items oii
                            INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
                            WHERE oi.order_id = od.order_id
                              AND oii.order_detail_id = od.id
                              AND oi.inspected_by_type = 'MARKET'
                            ORDER BY oi.inspection_round DESC
                            LIMIT 1
                        ), 0) AS market_inspected_quantity,
                        COALESCE((
                            SELECT oii.inspected_qty
                            FROM order_inspection_items oii
                            INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
                            WHERE oi.order_id = od.order_id
                              AND oii.order_detail_id = od.id
                              AND oi.inspected_by_type = 'CUSTOMER'
                            ORDER BY oi.inspection_round DESC
                            LIMIT 1
                        ), 0) AS customer_inspected_quantity
                    FROM order_details od
                    LEFT JOIN order_inspections oi_market ON od.order_id = oi_market.order_id AND oi_market.inspected_by_type = 'MARKET'
                    LEFT JOIN order_inspection_items oii_market ON oi_market.id = oii_market.inspection_id AND od.id = oii_market.order_detail_id
                    WHERE od.order_id = ?
                    GROUP BY od.id, od.product_code, od.product_name, od.category_id, od.category_name,
                             od.unit, od.ordered_qty, od.unit_price, od.discount_rate, od.discounted_unit_price,
                             od.ordered_amount, od.net_amount, od.processing_requirements,
                             od.remark
                    "#,
                    order_base.id
                )
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get order details for customer"))?;

                (order_base, order_items)
            }
        };


        // 查询退货换货记录（所有租户类型都返回）
        // 按 return_exchange_records.id 分组，每个 receipt 包含多个 items
        let receipt_rows = sqlx::query!(r#"
        SELECT 
            re.id AS receipt_id,
            re.operation_type,
            re.status,
            re.created_at,
            od.product_code,
            od.product_name,
            re.quantity,
            od.unit,
            re.reason
        FROM return_exchange_records re
        JOIN order_details od ON re.order_detail_id = od.id
        JOIN orders o ON od.order_id = o.id
        WHERE o.order_code = ?
        ORDER BY re.created_at DESC, re.id DESC
        "#,
            order_code
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get receipt records"))?;

        // 按 receipt_id 分组，组装成 ReceiptResponse 格式
        use std::collections::HashMap;
        let mut receipts_map: HashMap<i32, ReceiptResponse> = HashMap::new();

        for row in receipt_rows {
            let operation_type = ReceiptOperationType::try_from(row.operation_type.clone())
                .map_err(|e| AppError::Validation(format!("无效的收据操作类型: {}", e)))?;

            let receipt_item = ReceiptItem {
                product_id: row.product_code,
                product_name: row.product_name,
                quantity: row.quantity,
                unit: row.unit,
                reason: row.reason,
            };

            receipts_map
                .entry(row.receipt_id)
                .and_modify(|receipt| {
                    receipt.items.push(receipt_item.clone());
                })
                .or_insert_with(|| ReceiptResponse {
                    id: row.receipt_id,
                    operation_type,
                    status: row.status,
                    items: vec![receipt_item],
                    created_at: row.created_at,
                });
        }

        // 转换为 Vec 并保持排序（按创建时间倒序）
        let mut receipts: Vec<ReceiptResponse> = receipts_map.into_values().collect();
        receipts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(Some(OrderDetailResponse {
            order: order_base,
            items: order_items,
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
