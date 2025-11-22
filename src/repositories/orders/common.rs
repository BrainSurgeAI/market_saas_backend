use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::order::{
        CustomerOrderDetailResponse, CustomerOrderItem, CustomerOrderRound,
        ExchangeAndReturnOrderDetailResponse, MarketOrderDetailResponse, MarketOrderItem,
        MarketOrderRound, OrderDetail, OrderDetailResponse, OrderDelivery, OrderDeliveryItem,
        OrderQueryParams, OrderResponse, OrderStatusHistory, ProviderOrderItem,
        ProviderOrderRemark, ProviderOrderResponse, ProviderOrderRound, ReceiptItem,
        ReceiptOperationType, ReceiptResponse,
    },
    map_db_err,
    models::{claims::Claims, tenant_type::TenantType},
};
use async_trait::async_trait;
use sqlx::{MySql, QueryBuilder};
use tracing::debug;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use rust_decimal::Decimal;

// 查询所有 inspection items
#[derive(Debug, sqlx::FromRow)]
struct InspectionItemRow {
    inspection_id: u32,
    order_detail_id: i32,
    inspected_qty: Decimal,
    remarks: Option<String>,
}

// 查询订单的发货记录
#[derive(Debug, sqlx::FromRow)]
struct DeliveryRow {
    id: u64,
    assignment_id: u32,
    parent_id: Option<u64>,
    delivery_round: i32,
    delivery_type: String,
    delivered_at: Option<chrono::NaiveDateTime>,
    delivered_by: String,
    delivery_status: String,
    remark: Option<String>,
    delivery_contact_number: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

        // 查询订单的验收记录
// 先查询所有 inspections
#[derive(Debug, sqlx::FromRow)]
struct InspectionRow {
    id: u32,
    parent_id: Option<u32>,
    inspection_round: i32,
    inspected_by_type: String,
    inspected_by_id: i32,
    inspection_result: String,
    inspected_at: Option<chrono::NaiveDateTime>,
}

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

    async fn order_by_order_code_for_provider(
        &self,
        order_code: &str,
        provider_hash: &str,
    ) -> Result<Option<ProviderOrderResponse>, AppError>;

    async fn order_by_order_code_for_market(
        &self,
        order_code: &str,
        market_hash: &str,
    ) -> Result<Option<MarketOrderDetailResponse>, AppError>;

    async fn get_exchange_and_return_order_details_by_order_code(
        &self,
        order_code: &str,
    ) -> Result<Vec<ExchangeAndReturnOrderDetailResponse>, AppError>;

    async fn order_by_order_code_for_customer(
        &self,
        order_code: &str,
        customer_hash: &str,
    ) -> Result<Option<CustomerOrderDetailResponse>, AppError>;
}

#[async_trait]
impl CommonOrderRepository for MySqlRepository {

    async fn order_by_order_code_for_provider(
        &self,
        order_code: &str,
        provider_hash: &str,
    ) -> Result<Option<ProviderOrderResponse>, AppError> {
        // 获取订单基本信息和供应商ID
        let order_info = sqlx::query!(
            r#"
            SELECT o.order_code, o.order_status, o.created_at,
                   poa.provider_id, poa.id as assignment_id
            FROM orders o
            INNER JOIN provider_orders_assignments poa ON o.id = poa.order_id
            INNER JOIN tenants t ON poa.provider_id = t.id
            WHERE o.order_code = ? AND t.name_hash = ?
            "#,
            order_code,
            provider_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order basic info"))?;

        let order_info = match order_info {
            Some(info) => info,
            None => return Ok(None),
        };

        // 获取订单详细信息
        let order_detail = sqlx::query!(
            r#"
            SELECT o.delivery_date, o.confirmed_by AS receiver_name, t.name AS customer_name,
                   o.ordered_amount, o.discount_amount, o.net_amount,
                   o.market_contact_number AS receiver_phone, t.address AS delivery_address,
                   pd.delivered_by AS shipper_name, pd.delivery_contact_number AS shipper_phone
            FROM orders o
            INNER JOIN provider_orders_assignments poa ON poa.order_id = o.id
            LEFT JOIN provider_deliveries pd ON pd.assignment_id = poa.id AND pd.delivery_round = (
                SELECT MAX(delivery_round) FROM provider_deliveries WHERE assignment_id = poa.id
            )
            INNER JOIN tenants t ON o.market_id = t.id
            WHERE o.order_code = ? AND poa.provider_id = ?
            "#,
            order_code,
            order_info.provider_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order detail info"))?;

        // 获取所有轮次的发货信息
        let delivery_rounds = sqlx::query!(
            r#"
            SELECT pd.delivery_round, pd.delivery_status, pd.delivery_type,
                   pd.id as delivery_id
            FROM provider_deliveries pd
            WHERE pd.assignment_id = ?
            ORDER BY pd.delivery_round ASC
            "#,
            order_info.assignment_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get delivery rounds"))?;

        let mut rounds = Vec::new();

        // 如果没有发货记录，创建一个默认的轮次
        if delivery_rounds.is_empty() {
            // 获取所有订单商品项信息（没有发货和验收记录的情况）
            let items = sqlx::query!(
                r#"
                SELECT od.id as order_detail_id, od.product_code, od.product_name,
                       od.category_id, od.category_name, od.ordered_qty,
                       od.discounted_unit_price, od.unit, od.processing_requirements
                FROM order_details od
                WHERE od.order_id = (SELECT id FROM orders WHERE order_code = ?)
                "#,
                order_code
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get order details"))?;

            let mut round_items = Vec::new();

            for item in items {
                let provider_item = ProviderOrderItem {
                    order_detail_id: item.order_detail_id,
                    product_code: item.product_code,
                    product_name: item.product_name,
                    category_id: item.category_id,
                    category_name: item.category_name,
                    need_to_deliver_qty: item.ordered_qty, // 第一轮使用 ordered_qty
                    actual_qty: None, // 没有发货记录
                    unit_price: item.discounted_unit_price,
                    unit: item.unit,
                    inspection_status: "PENDING".to_string(), // 没有验收记录，默认为 PENDING
                    accepted_qty: None, // 没有验收记录
                    exchange_qty: None, // 没有换货记录
                    processing_requirements: item.processing_requirements,
                    remark: None, // 没有退换货记录
                };

                round_items.push(provider_item);
            }

            // 创建默认轮次
            let round = ProviderOrderRound {
                round: 1, // 默认第一轮
                delivery_status: "PENDING".to_string(), // 没有发货记录，默认为 PENDING
                delivery_type: "NORMAL".to_string(), // 默认正常发货
                items: round_items,
            };

            rounds.push(round);
        } else {
            // 有发货记录，正常处理每个轮次
            for round_info in delivery_rounds {
                // 获取该轮次的商品项信息
                let items = sqlx::query!(
                    r#"
                WITH order_details_info AS (
                    SELECT od.id as order_detail_id, od.product_code, od.product_name,
                           od.category_id, od.category_name, od.ordered_qty,
                           od.discounted_unit_price, od.unit, od.processing_requirements
                    FROM order_details od
                    WHERE od.order_id = (SELECT id FROM orders WHERE order_code = ?)
                ),
                    inspection_info AS (
                        SELECT oii.order_detail_id, oii.accepted, oii.remarks, oii.inspected_qty
                        FROM order_inspection_items oii
                        WHERE oii.inspection_id IN (
                            SELECT oi.id FROM order_inspections oi
                            WHERE oi.order_id = (SELECT id FROM orders WHERE order_code = ?)
                        )
                    ),
                    exchange_info AS (
                        SELECT rer.order_detail_id, rer.operation_type, rer.quantity,
                               rer.reason, rer.evidence_images
                        FROM return_exchange_records rer
                        WHERE rer.inspection_id IN (
                            SELECT oi.id FROM order_inspections oi
                            WHERE oi.order_id = (SELECT id FROM orders WHERE order_code = ?)
                        )
                    ),
                    delivery_info AS (
                        SELECT pdi.order_detail_id, pdi.actual_qty
                        FROM provider_delivery_items pdi
                        WHERE pdi.delivery_id = ?
                    )
                SELECT
                    odi.order_detail_id,
                    odi.product_code,
                    odi.product_name,
                    odi.category_id,
                    odi.category_name,
                    odi.ordered_qty,
                    odi.discounted_unit_price,
                    odi.unit,
                    odi.processing_requirements,
                    ii.accepted,
                    ii.remarks,
                    ii.inspected_qty as accepted_qty,
                    ei.operation_type,
                    ei.quantity as exchange_quantity,
                    ei.reason,
                    ei.evidence_images,
                    di.actual_qty as delivered_qty
                FROM order_details_info odi
                    LEFT JOIN inspection_info ii ON odi.order_detail_id = ii.order_detail_id
                    LEFT JOIN exchange_info ei ON odi.order_detail_id = ei.order_detail_id
                    LEFT JOIN delivery_info di ON odi.order_detail_id = di.order_detail_id
                    "#,
                    order_code,
                    order_code,
                    order_code,
                    round_info.delivery_id
                )
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_err!("Failed to get round items"))?;

                let mut round_items = Vec::new();

                for item in items {
                    // 计算 needToDeliverQty：在第一轮是 ordered_qty，否则是 exchange_quantity
                    let need_to_deliver_qty = if round_info.delivery_round == 1 {
                        item.ordered_qty
                    } else {
                        item.exchange_quantity.unwrap_or(Decimal::ZERO)
                    };

                    // 计算 inspectionStatus
                    let inspection_status = if item.accepted == Some(1) && item.remarks.is_some() {
                        "SIGN".to_string()
                    } else if let Some(operation_type) = item.operation_type {
                        operation_type
                    } else {
                        "PENDING".to_string()
                    };

                    // 构建 remark
                    let remark = if item.reason.is_some() || item.evidence_images.is_some() {
                        Some(ProviderOrderRemark {
                            evidence_images: item.evidence_images,
                            reason: item.reason,
                        })
                    } else {
                        None
                    };

                    let provider_item = ProviderOrderItem {
                        order_detail_id: item.order_detail_id,
                        product_code: item.product_code,
                        product_name: item.product_name,
                        category_id: item.category_id,
                        category_name: item.category_name,
                        need_to_deliver_qty,
                        actual_qty: item.delivered_qty,
                        unit_price: item.discounted_unit_price,
                        unit: item.unit,
                        inspection_status,
                        accepted_qty: item.accepted_qty,
                        exchange_qty: item.exchange_quantity,
                        processing_requirements: item.processing_requirements,
                        remark,
                    };

                    round_items.push(provider_item);
                }

                let round = ProviderOrderRound {
                    round: round_info.delivery_round,
                    delivery_status: round_info.delivery_status,
                    delivery_type: round_info.delivery_type,
                    items: round_items,
                };

                rounds.push(round);
            }
        }

        let (delivery_date, receiver_name, customer_name, ordered_amount, discount_amount,
             net_amount, receiver_phone, delivery_address, shipper_name, shipper_phone) =
            if let Some(detail) = order_detail {
                (
                    Some(DateTime::<Utc>::from_naive_utc_and_offset(detail.delivery_date.into(), Utc)),
                    detail.receiver_name,
                    Some(detail.customer_name),
                    detail.ordered_amount,
                    detail.discount_amount,
                    detail.net_amount,
                    detail.receiver_phone,
                    Some(detail.delivery_address),
                    detail.shipper_name,
                    detail.shipper_phone,
                )
            } else {
                (None, None, None, Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, None, None, None, None)
            };

        let response = ProviderOrderResponse {
            order_code: order_info.order_code,
            order_status: order_info.order_status,
            delivery_date,
            receiver_name,
            customer_name,
            ordered_amount,
            discount_amount,
            net_amount,
            receiver_phone,
            delivery_address,
            shipper_name,
            shipper_phone,
            rounds,
            created_at: order_info.created_at,
        };

        Ok(Some(response))
    }

    async fn order_by_order_code_for_market(
        &self,
        order_code: &str,
        market_hash: &str,
    ) -> Result<Option<MarketOrderDetailResponse>, AppError> {
        // 获取订单基本信息
        let order_info = sqlx::query!(
            r#"
            SELECT o.id as order_id, o.order_code, o.order_status, o.created_at, o.updated_at,
                   o.ordered_amount, o.discount_amount, o.net_amount, o.delivery_date,
                   o.delivery_address, o.contact_name as receiver_name, o.contact_phone as receiver_phone,
                   mt.name AS market_name, pt.name AS provider_name,
                   ds.name AS shipper_name, ds.phone AS shipper_phone
            FROM orders o
            INNER JOIN tenants mt ON o.market_id = mt.id
            INNER JOIN tenants pt ON o.customer_id = pt.id
            LEFT JOIN delivery_staff ds ON o.delivery_staff_id = ds.id
            WHERE o.order_code = ? AND mt.name_hash = ?
            "#,
            order_code,
            market_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order basic info"))?;

        let order_info = match order_info {
            Some(info) => info,
            None => return Ok(None),
        };

        // 获取所有轮次的发货信息
        let delivery_rounds = sqlx::query!(
            r#"
            SELECT pd.delivery_round, pd.delivery_type, pd.delivery_status,
                   pd.delivered_at, oi.inspection_result, oi.inspected_at
            FROM provider_deliveries pd
            INNER JOIN provider_orders_assignments poa ON pd.assignment_id = poa.id
            INNER JOIN orders o ON poa.order_id = o.id
            LEFT JOIN order_inspections oi ON oi.order_id = o.id
                AND oi.inspection_round = pd.delivery_round
                AND oi.inspected_by_type = 'MARKET'
            WHERE o.order_code = ?
            ORDER BY pd.delivery_round ASC
            "#,
            order_code
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get delivery rounds"))?;

        let mut rounds_response = Vec::new();

        for round_info in delivery_rounds {
            // 获取该轮次的商品验收信息
            let round_items = sqlx::query!(
                r#"
                SELECT od.id as order_detail_id, od.product_code, od.product_name,
                       od.category_id, od.category_name, od.unit, od.discounted_unit_price as unit_price,
                       od.ordered_qty, COALESCE(pdi.actual_qty, 0) as need_to_inspect_qty,
                       oii.inspected_qty as accepted_qty,
                       CASE
                           WHEN oii.result = 'SIGN' THEN oii.inspected_qty
                           ELSE 0
                       END as accepted_qty_calc,
                       CASE
                           WHEN oii.result = 'EXCHANGE' THEN oii.inspected_qty
                           ELSE 0
                       END as exchange_qty,
                       CASE
                           WHEN oii.result = 'RETURN' THEN oii.inspected_qty
                           ELSE 0
                       END as return_qty,
                       CASE
                           WHEN oii.result = 'SIGN' THEN 'SIGN'
                           WHEN oii.result = 'EXCHANGE' THEN 'EXCHANGE'
                           WHEN oii.result = 'RETURN' THEN 'RETURN'
                           ELSE 'PENDING'
                       END as inspection_status,
                       od.processing_requirements,
                       CASE
                           WHEN oii.result IN ('EXCHANGE', 'RETURN') THEN oii.remarks
                           ELSE NULL
                       END as remarks
                FROM order_details od
                INNER JOIN orders o ON od.order_id = o.id
                LEFT JOIN provider_delivery_items pdi ON od.id = pdi.order_detail_id
                    AND pdi.delivery_id IN (
                        SELECT pd.id FROM provider_deliveries pd
                        INNER JOIN provider_orders_assignments poa ON pd.assignment_id = poa.id
                        WHERE poa.order_id = o.id AND pd.delivery_round = ?
                    )
                LEFT JOIN order_inspection_items oii ON od.id = oii.order_detail_id
                    AND oii.inspection_id IN (
                        SELECT oi.id FROM order_inspections oi
                        WHERE oi.order_id = o.id AND oi.inspection_round = ?
                        AND oi.inspected_by_type = 'MARKET'
                    )
                WHERE o.order_code = ?
                "#,
                round_info.delivery_round,
                round_info.delivery_round,
                order_code
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get round items"))?;

            let mut items_response = Vec::new();

            for item in round_items {
                let remark = if item.remarks.is_some() {
                    Some(ProviderOrderRemark {
                        evidence_images: None, // TODO: 如果需要添加证据图片字段，需要修改数据库结构
                        reason: item.remarks,
                    })
                } else {
                    None
                };

                let market_item = MarketOrderItem {
                    order_detail_id: item.order_detail_id as i32,
                    product_code: item.product_code,
                    product_name: item.product_name,
                    category_id: item.category_id,
                    category_name: item.category_name,
                    unit: item.unit,
                    unit_price: item.unit_price,
                    ordered_qty: item.ordered_qty,
                    need_to_inspect_qty: item.need_to_inspect_qty,
                    accepted_qty: item.accepted_qty,
                    exchange_qty: if item.exchange_qty > Some(Decimal::ZERO) { item.exchange_qty } else { None },
                    return_qty: if item.return_qty > Some(Decimal::ZERO) { item.return_qty } else { None },
                    inspection_status: item.inspection_status,
                    processing_requirements: item.processing_requirements,
                    remark,
                };

                items_response.push(market_item);
            }

            let market_round = MarketOrderRound {
                round: round_info.delivery_round,
                delivery_type: round_info.delivery_type,
                delivered_at: round_info.delivered_at.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
                inspection_result: round_info.inspection_result.unwrap_or_else(|| "PENDING".to_string()),
                inspection_at: round_info.inspected_at.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
                items: items_response,
            };

            rounds_response.push(market_round);
        }

        // 获取订单的商品基本信息
        let order_details = sqlx::query_as!(
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
                od.remark
            FROM order_details od
            WHERE od.order_id = ?
            ORDER BY od.id ASC
            "#,
            order_info.order_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order details"))?;

        let response = MarketOrderDetailResponse {
            order_code: order_info.order_code,
            order_status: order_info.order_status,
            created_at: order_info.created_at,
            customer_name: Some(order_info.market_name),
            delivery_address: Some(order_info.delivery_address),
            delivery_date: Some(DateTime::<Utc>::from_naive_utc_and_offset(order_info.delivery_date.into(), Utc)),
            discount_amount: order_info.discount_amount,
            net_amount: order_info.net_amount,
            ordered_amount: order_info.ordered_amount,
            receiver_name: Some(order_info.receiver_name),
            receiver_phone: Some(order_info.receiver_phone),
            shipper_name: order_info.shipper_name,
            shipper_phone: order_info.shipper_phone,
            details: order_details,
            rounds: rounds_response,
        };

        Ok(Some(response))
    }

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
                            od.remark
                        FROM order_details od
                        INNER JOIN provider_orders_assignments poa 
                            ON od.order_id = poa.order_id AND poa.provider_id = ?
                        WHERE od.order_id = ?
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
                        od.remark
                    FROM order_details od
                    WHERE od.order_id = ?
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
                        od.remark
                    FROM order_details od
                    WHERE od.order_id = ?
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
            re.inspection_id,
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
                inspection_id: row.inspection_id,
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
                    inspection_id: row.inspection_id,
                    operation_type,
                    status: row.status,
                    items: vec![receipt_item],
                    created_at: row.created_at,
                });
        }

        // 转换为 Vec 并保持排序（按创建时间倒序）
        let mut receipts: Vec<ReceiptResponse> = receipts_map.into_values().collect();
        receipts.sort_by(|a, b| b.created_at.cmp(&a.created_at));



        let inspection_rows = sqlx::query_as!(
            InspectionRow,
            r#"
            SELECT 
                id,
                parent_id,
                inspection_round,
                inspected_by_type,
                inspected_by_id,
                inspection_result,
                inspected_at
            FROM order_inspections
            WHERE order_id = ?
            ORDER BY inspection_round ASC, inspected_at ASC
            "#,
            order_base.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order inspections"))?;



        let inspection_item_rows = sqlx::query_as!(
            InspectionItemRow,
            r#"
            SELECT 
                oii.inspection_id,
                oii.order_detail_id,
                oii.inspected_qty,
                oii.remarks
            FROM order_inspection_items oii
            INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
            WHERE oi.order_id = ?
            "#,
            order_base.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order inspection items"))?;

        // 构建 inspection_id -> items 的映射
        let mut inspection_items_map: HashMap<u32, Vec<crate::dto::order::OrderInspectionItem>> = HashMap::new();
        for item_row in inspection_item_rows {
            let inspection_id = item_row.inspection_id;
            let inspection = inspection_rows.iter().find(|r| r.id == inspection_id);
            
            let inspection_item = if let Some(_) = inspection {
                // if insp.inspection_result == "PENDING" {
                //     crate::dto::order::OrderInspectionItem {
                //         order_detail_id: item_row.order_detail_id,
                //         inspected_qty: None,
                //         quantity: Some(item_row.inspected_qty),
                //         remark: item_row.remarks,
                //     }
                // } else {
                    // 非 PENDING 状态：使用 inspectedQty 和 remark
                    crate::dto::order::OrderInspectionItem {
                        order_detail_id: item_row.order_detail_id,
                        inspected_qty: Some(item_row.inspected_qty),
                        quantity: None,
                        remark: item_row.remarks,
                    }
                //}
            } 
            else {
                // 如果没有找到对应的 inspection，使用默认值
                crate::dto::order::OrderInspectionItem {
                    order_detail_id: item_row.order_detail_id,
                    inspected_qty: Some(item_row.inspected_qty),
                    quantity: None,
                    remark: item_row.remarks,
                }
            };

            inspection_items_map
                .entry(inspection_id)
                .or_insert_with(Vec::new)
                .push(inspection_item);
        }

        // 组装 inspections
        let mut inspections: Vec<crate::dto::order::OrderInspection> = Vec::new();
        for row in inspection_rows {
            // 只获取已验收的商品 items（如果 inspection_items 表中有记录）
            let items = inspection_items_map
                .get(&row.id)
                .cloned()
                .unwrap_or_default();

            let inspection = crate::dto::order::OrderInspection {
                inspection_id: row.id as i32,
                parent_id: row.parent_id.map(|p| p as i32),
                inspection_round: row.inspection_round,
                inspected_by_type: row.inspected_by_type,
                inspected_by_id: row.inspected_by_id,
                result: row.inspection_result,
                inspected_at: row.inspected_at.map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
                items,
            };

            inspections.push(inspection);
        }


        let delivery_rows = sqlx::query_as!(
            DeliveryRow,
            r#"
            SELECT 
                pd.id,
                pd.assignment_id,
                pd.parent_id,
                pd.delivery_round,
                pd.delivery_type,
                pd.delivered_at,
                pd.delivered_by,
                pd.delivery_status,
                pd.remark,
                pd.delivery_contact_number,
                pd.created_at,
                pd.updated_at
            FROM provider_deliveries pd
            INNER JOIN provider_orders_assignments poa ON pd.assignment_id = poa.id
            WHERE poa.order_id = ?
            ORDER BY pd.delivery_round ASC, pd.created_at ASC
            "#,
            order_base.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order deliveries"))?;

        // 查询所有 delivery items
        #[derive(Debug, sqlx::FromRow)]
        struct DeliveryItemRow {
            id: u64,
            delivery_id: u64,
            order_detail_id: u64,
            product_code: String,
            actual_qty: Decimal,
            unit_price: Decimal,
            subtotal: Option<Decimal>,
            weight_unit: Option<String>,
            remark: Option<String>,
            created_at: chrono::NaiveDateTime,
            updated_at: chrono::NaiveDateTime,
        }

        let delivery_item_rows = sqlx::query_as!(
            DeliveryItemRow,
            r#"
            SELECT 
                pdi.id,
                pdi.delivery_id,
                pdi.order_detail_id,
                pdi.product_code,
                pdi.actual_qty,
                pdi.unit_price,
                pdi.subtotal,
                pdi.weight_unit,
                pdi.remark,
                pdi.created_at,
                pdi.updated_at
            FROM provider_delivery_items pdi
            INNER JOIN provider_deliveries pd ON pdi.delivery_id = pd.id
            INNER JOIN provider_orders_assignments poa ON pd.assignment_id = poa.id
            WHERE poa.order_id = ?
            "#,
            order_base.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order delivery items"))?;

        // 构建 delivery_id -> items 的映射
        let mut delivery_items_map: HashMap<u64, Vec<OrderDeliveryItem>> = HashMap::new();
        for item_row in delivery_item_rows {
            let delivery_item = OrderDeliveryItem {
                id: item_row.id as i64,
                order_detail_id: item_row.order_detail_id as i64,
                product_code: item_row.product_code,
                actual_qty: item_row.actual_qty,
                unit_price: item_row.unit_price,
                subtotal: item_row.subtotal.unwrap_or(item_row.actual_qty * item_row.unit_price),
                weight_unit: item_row.weight_unit,
                remark: item_row.remark,
                created_at: DateTime::from_naive_utc_and_offset(item_row.created_at, Utc),
                updated_at: DateTime::from_naive_utc_and_offset(item_row.updated_at, Utc),
            };

            delivery_items_map
                .entry(item_row.delivery_id)
                .or_insert_with(Vec::new)
                .push(delivery_item);
        }

        // 组装 deliveries
        let mut deliveries: Vec<OrderDelivery> = Vec::new();
        for row in delivery_rows {
            let items = delivery_items_map
                .get(&row.id)
                .cloned()
                .unwrap_or_default();

            let delivery = OrderDelivery {
                id: row.id as i64,
                assignment_id: row.assignment_id,
                parent_id: row.parent_id.map(|p| p as i64),
                delivery_round: row.delivery_round,
                delivery_type: row.delivery_type,
                delivered_at: row.delivered_at.map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
                delivered_by: row.delivered_by,
                delivery_status: row.delivery_status,
                remark: row.remark,
                delivery_contact_number: row.delivery_contact_number,
                created_at: DateTime::from_naive_utc_and_offset(row.created_at, Utc),
                updated_at: DateTime::from_naive_utc_and_offset(row.updated_at, Utc),
                items,
            };

            deliveries.push(delivery);
        }

        // 查询订单状态变更历史
        #[derive(Debug, sqlx::FromRow)]
        struct StatusHistoryRow {
            to_status: String,
            change_reason: Option<String>,
            created_at: DateTime<Utc>,
        }

        let status_history_rows = sqlx::query_as!(
            StatusHistoryRow,
            r#"
            SELECT 
                to_status,
                change_reason,
                created_at
            FROM order_status_history
            WHERE order_id = ?
            ORDER BY created_at ASC
            "#,
            order_base.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order status history"))?;

        let status_history: Vec<OrderStatusHistory> = status_history_rows
            .into_iter()
            .map(|row| OrderStatusHistory {
                to_status: row.to_status,
                change_reason: row.change_reason,
                created_at: row.created_at,
            })
            .collect();

        Ok(Some(OrderDetailResponse {
            order: order_base,
            items: order_items,
            after_sales: receipts,
            inspections,
            deliveries,
            status_history,
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

    async fn order_by_order_code_for_customer(
        &self,
        order_code: &str,
        customer_hash: &str,
    ) -> Result<Option<CustomerOrderDetailResponse>, AppError> {
        // 获取订单基本信息
        let order_info = sqlx::query!(
            r#"
            SELECT o.id as order_id, o.order_code, o.order_status, o.created_at, o.updated_at,
                   o.ordered_amount, o.discount_amount, o.net_amount, o.delivery_date,
                   o.delivery_address, o.contact_name as receiver_name, o.contact_phone as receiver_phone,
                   pt.name AS customer_name, mt.name AS market_name,
                   ds.name AS shipper_name, ds.phone AS shipper_phone
            FROM orders o
            INNER JOIN tenants pt ON o.customer_id = pt.id
            INNER JOIN tenants mt ON o.market_id = mt.id
            LEFT JOIN delivery_staff ds ON o.delivery_staff_id = ds.id
            WHERE o.order_code = ? AND pt.name_hash = ?
            "#,
            order_code,
            customer_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order basic info"))?;

        let order_info = match order_info {
            Some(info) => info,
            None => return Ok(None),
        };

        // 获取市场验收通过（PASS 或 PARTIAL）的轮次
        let market_passed_rounds = sqlx::query!(
            r#"
            SELECT oi.inspection_round
            FROM order_inspections oi
            WHERE oi.order_id = ? AND oi.inspected_by_type = 'MARKET'
                  AND oi.inspection_result IN ('PASS', 'PARTIAL')
            ORDER BY oi.inspection_round ASC
            "#,
            order_info.order_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get market passed rounds"))?;

        let mut rounds_response = Vec::new();

        for round_info in market_passed_rounds {
            // 获取该轮次的客户验收信息（inspectionResult 和 inspectionAt）
            let customer_inspection = sqlx::query!(
                r#"
                SELECT oi.inspection_result, oi.inspected_at
                FROM order_inspections oi
                WHERE oi.order_id = ? AND oi.inspection_round = ? AND oi.inspected_by_type = 'CUSTOMER'
                "#,
                order_info.order_id,
                round_info.inspection_round
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get customer inspection"))?;

            // 如果没有客户验收记录，显示为 PENDING
            let (inspection_result, inspection_at) = customer_inspection
                .map(|ci| (ci.inspection_result, ci.inspected_at))
                .unwrap_or_else(|| ("PENDING".to_string(), None));

            // 获取该轮次的商品验收信息（来自 MARKET 验收记录）
            let round_items = sqlx::query!(
                r#"
                SELECT od.id as order_detail_id, od.product_code, od.product_name,
                       od.category_id, od.category_name, od.unit, od.discounted_unit_price as unit_price,
                       od.ordered_qty, od.processing_requirements,
                       -- CUSTOMER 的验收信息
                       customer_oii.inspected_qty as customer_accepted_qty,
                       COALESCE(
                           CASE
                               WHEN customer_oii.result = 'SIGN' THEN 'SIGN'
                               WHEN customer_oii.result = 'EXCHANGE' THEN 'EXCHANGE'
                               WHEN customer_oii.result = 'RETURN' THEN 'RETURN'
                               ELSE NULL
                           END,
                           'PENDING'
                       ) as customer_inspection_status,
                       customer_oii.remarks as customer_remarks,
                       -- MARKET 的验收信息（用于计算 exchange_qty 和 return_qty）
                       market_oii.inspected_qty as market_inspected_qty,
                       market_oii.result as market_result
                FROM order_details od
                INNER JOIN orders o ON od.order_id = o.id
                -- 连接 MARKET 验收记录
                LEFT JOIN order_inspection_items market_oii ON od.id = market_oii.order_detail_id
                    AND market_oii.inspection_id IN (
                        SELECT oi.id FROM order_inspections oi
                        WHERE oi.order_id = o.id AND oi.inspection_round = ?
                        AND oi.inspected_by_type = 'MARKET'
                    )
                -- 连接 CUSTOMER 验收记录
                LEFT JOIN order_inspection_items customer_oii ON od.id = customer_oii.order_detail_id
                    AND customer_oii.inspection_id IN (
                        SELECT oi.id FROM order_inspections oi
                        WHERE oi.order_id = o.id
                        AND oi.inspected_by_type = 'CUSTOMER'
                    )
                WHERE o.order_code = ? AND (market_oii.result IS NULL OR market_oii.result != 'RETURN')
                "#,
                round_info.inspection_round,
                order_code
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get round items"))?;

            // 计算 needToInspectQty：聚合 MARKET 验收时 accepted=1 的记录中相同 order_detail_id 的 inspected_qty
            let need_to_inspect_map = sqlx::query!(
                r#"
                SELECT oii.order_detail_id, SUM(oii.inspected_qty) as total_inspected_qty
                FROM order_inspection_items oii
                INNER JOIN order_inspections oi ON oii.inspection_id = oi.id
                WHERE oi.order_id = ? AND oi.inspected_by_type = 'MARKET' AND oii.accepted = 1
                GROUP BY oii.order_detail_id
                "#,
                order_info.order_id
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get need to inspect quantities"))?;

            // 创建 order_detail_id -> need_to_inspect_qty 的映射
            let need_to_inspect_map: std::collections::HashMap<i32, Decimal> = need_to_inspect_map
                .into_iter()
                .map(|row| (row.order_detail_id, row.total_inspected_qty.unwrap_or(Decimal::ZERO)))
                .collect();

            let mut items_response = Vec::new();

            for item in round_items {
                // 计算 exchange_qty 和 return_qty（基于 MARKET 的验收结果）
                let (exchange_qty, return_qty) = match item.market_result.as_deref() {
                    Some("EXCHANGE") => (item.market_inspected_qty, None),
                    Some("RETURN") => (None, item.market_inspected_qty),
                    _ => (None, None),
                };

                let remark = if item.customer_remarks.is_some() {
                    Some(ProviderOrderRemark {
                        evidence_images: None, // TODO: 如果需要添加证据图片字段，需要修改数据库结构
                        reason: item.customer_remarks,
                    })
                } else {
                    None
                };

                let customer_item = CustomerOrderItem {
                    order_detail_id: item.order_detail_id,
                    product_code: item.product_code,
                    product_name: item.product_name,
                    category_id: item.category_id,
                    category_name: item.category_name,
                    unit: item.unit,
                    unit_price: item.unit_price,
                    ordered_qty: item.ordered_qty,
                    need_to_inspect_qty: need_to_inspect_map.get(&item.order_detail_id).cloned(),
                    accepted_qty: item.customer_accepted_qty,
                    exchange_qty,
                    return_qty,
                    inspection_status: item.customer_inspection_status,
                    processing_requirements: item.processing_requirements,
                    remark,
                };

                items_response.push(customer_item);
            }

            let customer_round = CustomerOrderRound {
                round: round_info.inspection_round,
                inspection_result,
                inspection_at: inspection_at.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
                items: items_response,
            };

            rounds_response.push(customer_round);
        }

        // 获取订单的商品基本信息
        let order_details = sqlx::query_as!(
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
                od.remark
            FROM order_details od
            WHERE od.order_id = ?
            ORDER BY od.id ASC
            "#,
            order_info.order_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order details"))?;

        let response = CustomerOrderDetailResponse {
            order_code: order_info.order_code,
            order_status: order_info.order_status,
            created_at: order_info.created_at,
            customer_name: Some(order_info.customer_name),
            delivery_address: Some(order_info.delivery_address),
            delivery_date: Some(DateTime::<Utc>::from_naive_utc_and_offset(order_info.delivery_date.into(), Utc)),
            discount_amount: order_info.discount_amount,
            net_amount: order_info.net_amount,
            ordered_amount: order_info.ordered_amount,
            receiver_name: Some(order_info.receiver_name),
            receiver_phone: Some(order_info.receiver_phone),
            shipper_name: order_info.shipper_name,
            shipper_phone: order_info.shipper_phone,
            details: order_details,
            rounds: rounds_response,
        };

        Ok(Some(response))
    }
}
