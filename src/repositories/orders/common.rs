use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::order::{
        CustomerOrderDetailResponse, CustomerOrderItem, CustomerOrderRound,
        ExchangeAndReturnOrderDetailResponse, MarketOrderDetailResponse, OrderDetail,
        OrderQueryParams, OrderResponse, ProviderOrderItem, ProviderOrderRemark,
        ProviderOrderResponse, ProviderOrderRound, ProviderReturnExchangeItem,
        ProviderReturnExchangeOrderResponse, MarketOrderStatisticsResponse,
        ProviderDashboardStatsDTO, DashboardStatsDTO,
    },
    map_db_err,
    models::{order_status::OrderStatus, tenant_type::TenantType},
};
use async_trait::async_trait;
use sqlx::{FromRow, MySql, QueryBuilder};
use tracing::debug;

use chrono::{DateTime, Local, NaiveDate, Utc};

use rust_decimal::Decimal;

// 用于查询配送统计的临时结构
#[derive(Debug, FromRow)]
struct DeliveryStatsRow {
    delivering_count: i64,
    completed_count: i64,
}

#[async_trait]
pub(crate) trait CommonOrderRepository: Send + Sync {
    async fn get_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &str,
        query_params: &OrderQueryParams,
    ) -> Result<Vec<OrderResponse>, AppError>;

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

    async fn get_provider_return_exchange_orders(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<crate::dto::order::ProviderReturnExchangeOrderResponse>, AppError>;

    /// Get market order statistics
    ///
    /// # Arguments
    /// * `tenant_hash` - The hash of the market tenant
    ///
    /// # Returns
    /// A result containing the market order statistics
    async fn get_market_dashboard_stats(
        &self,
        tenant_hash: &str,
    ) -> Result<MarketOrderStatisticsResponse, AppError>;


    async fn get_provider_dashboard_stats(
        &self,
        provider_hash: &str,
        delivery_date: Option<NaiveDate>,
    ) -> Result<ProviderDashboardStatsDTO, AppError>;

    async fn get_customer_dashboard_stats(
        &self,
        customer_hash: &str,
        date: Option<NaiveDate>,
    ) -> Result<DashboardStatsDTO, AppError>;
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
            SELECT o.id as order_id, o.order_code, o.order_status, o.delivery_date, o.delivery_address, 
                   o.confirmed_by as receiver_name, o.market_contact_number as receiver_phone, o.created_at,
                   mt.name as customer_name, o.ordered_amount, o.discount_amount, o.net_amount,
                   ds.name as shipper_name, ds.phone as shipper_phone,
                   o.market_contact_number, o.assigned_by as market_contactor_name
            FROM orders o
            INNER JOIN provider_orders_assignments poa ON o.id = poa.order_id
            INNER JOIN tenants t ON poa.provider_id = t.id
            INNER JOIN tenants mt ON o.market_id = mt.id
            LEFT JOIN delivery_staff ds ON o.delivery_staff_id = ds.id
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

        // 1 订单状态如果是ASSIGNED,从order_details表中获取信息构建对应的ProviderOrderRound
        // 2 订单状态如果是EXCHANGE_REQUESTED,从return_exchange_records表中获取信息构建对应的ProviderOrderRound；

        let (order_details, delivery_type): (Vec<ProviderOrderItem>, String) =
            match OrderStatus::try_from(order_info.order_status.as_str())? {
                OrderStatus::Assigned | OrderStatus::SupplierPreparing => {
                    // 从order_details表中获取信息构建ProviderOrderResponse
                    let items = sqlx::query!(
                        r#"
                        SELECT
                            od.id as order_detail_id,
                            od.product_code,
                            od.product_name,
                            od.category_id,
                            od.category_name,
                            od.unit,
                            od.ordered_qty as need_to_deliver_qty,
                            od.unit_price,
                            od.processing_requirements,
                            tiu.temp_url as image_url
                        FROM order_details od
                        INNER JOIN orders o ON od.order_id = o.id
                        LEFT JOIN temp_image_urls tiu ON od.product_code = tiu.product_code
                        WHERE od.order_id = ?
                        ORDER BY od.id ASC
                        "#,
                        order_info.order_id
                    )
                    .fetch_all(&self.pool)
                    .await
                    .map_err(map_db_err!("Failed to get order details"))?
                    .into_iter()
                    .map(|row| ProviderOrderItem {
                        order_detail_id: row.order_detail_id,
                        product_code: row.product_code,
                        product_name: row.product_name,
                        category_id: row.category_id,
                        category_name: row.category_name,
                        need_to_deliver_qty: row.need_to_deliver_qty,
                        actual_qty: None, // 总是None，因为还没有发货
                        unit_price: row.unit_price,
                        unit: row.unit,
                        processing_requirements: row.processing_requirements,
                        image_url: row.image_url,
                        remark: None
                    })
                    .collect();
                    (items, "NORMAL".to_string())
                }
                OrderStatus::ExchangeRequested | OrderStatus::ExchangeInProgress => {
                    // 从return_exchange_records表中获取信息构建ProviderOrderResponse

                    let items = sqlx::query!(
                        r#"
                        SELECT
                        rer.order_detail_id,
                        od.product_code,
                        od.product_name,
                        od.category_id,
                        od.category_name,
                        od.unit,
                        od.unit_price,
                        rer.quantity as need_to_deliver_qty,
                        od.processing_requirements,
                        rer.operation_type as last_inspection_result,
                        tiu.temp_url as image_url
                        FROM return_exchange_records rer
                        INNER JOIN order_details od ON rer.order_detail_id = od.id
                        LEFT JOIN temp_image_urls tiu ON od.product_code = tiu.product_code
                        WHERE rer.order_detail_id IN (SELECT order_detail_id FROM provider_deliveries WHERE order_id = ?)
                        AND rer.operation_type = 'EXCHANGE'
                        AND rer.inspection_id IN (
                            SELECT oi.id FROM order_inspections oi
                            WHERE oi.order_id = ?
                            AND oi.inspection_round = (
                                SELECT MAX(inspection_round) FROM order_inspections
                                WHERE order_id = ? AND inspection_result = 'EXCHANGE'
                            )
                            AND oi.inspection_result = 'EXCHANGE'
                        )
                        ORDER BY rer.order_detail_id ASC
                        "#,
                        order_info.order_id,
                        order_info.order_id,
                        order_info.order_id,
                    )
                    .fetch_all(&self.pool)
                    .await
                    .map_err(map_db_err!("Failed to get return exchange records"))?
                    .into_iter()
                    .map(|row| ProviderOrderItem {
                        order_detail_id: row.order_detail_id,
                        product_code: row.product_code,
                        product_name: row.product_name,
                        category_id: row.category_id,
                        category_name: row.category_name,
                        need_to_deliver_qty: row.need_to_deliver_qty,
                        actual_qty: None, // 总是None，因为还没有发货
                        unit_price: row.unit_price,
                        unit: row.unit,
                        processing_requirements: row.processing_requirements,
                        image_url: row.image_url,
                        remark: None
                    })
                    .collect();
                    (items, "EXCHANGE".to_string())
                }
                _ => {
                    (Vec::new(), "NORMAL".to_string())
                }
            };

        let response = ProviderOrderResponse {
            order_code: order_info.order_code,
            order_status: order_info.order_status,
            created_at: order_info.created_at,
            delivery_date: order_info.delivery_date,
            delivery_address: order_info.delivery_address,
            receiver_name: order_info.receiver_name,
            receiver_phone: order_info.receiver_phone,
            customer_name: order_info.customer_name,
            ordered_amount: order_info.ordered_amount,
            discount_amount: order_info.discount_amount,
            net_amount: order_info.net_amount,
            shipper_name: order_info.shipper_name,
            shipper_phone: order_info.shipper_phone,
            market_contact_number: order_info.market_contact_number,
            market_contactor_name: order_info.market_contactor_name,
            current: ProviderOrderRound {
                delivery_status: "PENDING".to_string(),
                delivery_type: delivery_type,
                items: order_details,
            },
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
        .map_err(map_db_err!("Failed to get order basic info"))?
        .ok_or_else(|| AppError::not_found(format!("Order not found: {}", order_code)))?;

        // 获取订单明细信息
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
                od.remark,
                od.accepted_qty,
                tiu.temp_url as image_url
            FROM order_details od
            LEFT JOIN temp_image_urls tiu ON od.product_code = tiu.product_code
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
            delivery_date: order_info.delivery_date,
            discount_amount: order_info.discount_amount,
            net_amount: order_info.net_amount,
            ordered_amount: order_info.ordered_amount,
            receiver_name: Some(order_info.receiver_name),
            receiver_phone: Some(order_info.receiver_phone),
            shipper_name: order_info.shipper_name,
            shipper_phone: order_info.shipper_phone,
            details: order_details,
            rounds: vec![], // TODO: 实现轮次信息
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

        let basic_query = r#"
                SELECT 
                   o.order_code, 
                   o.delivery_address, 
                   o.ordered_amount,
                   o.net_amount, 
                   o.delivery_date, 
                   o.order_status,
                   o.created_at,
                   mt.name as market_name,
                   mt.address as market_address,
                   pt.name as customer_name,
                   pt.address as customer_address,
                   o.contact_name,
                   o.contact_phone,
                   o.market_contact_number,
                   o.assigned_by as market_contactor_name,
                   o.urgent,
                   ds.name as shipper_name,
                   ds.phone as shipper_phone,
                   (SELECT COUNT(DISTINCT od.product_code) 
                    FROM order_details od 
                    WHERE od.order_id = o.id) as sku_count
                   FROM orders o
                   INNER JOIN tenants mt ON o.market_id = mt.id
                   INNER JOIN tenants pt ON o.customer_id = pt.id
                   LEFT JOIN delivery_staff ds ON o.delivery_staff_id = ds.id
                   {JOIN_CLAUSE} WHERE 1=1 "#;

        // Parse tenant_type string to enum for match
        let tenant_type_enum = TenantType::try_from(tenant_type)
            .map_err(|_| AppError::Validation(format!("Invalid tenant type: {}", tenant_type)))?;

        let mut builder: QueryBuilder<MySql>;
        match tenant_type_enum {
            TenantType::Provider => {
                let query = &basic_query.replace(
                    "{JOIN_CLAUSE}",
                    "JOIN provider_orders_assignments po ON po.order_id=o.id",
                );

                builder = QueryBuilder::new(query);
                builder.push(" AND po.provider_id= ").push_bind(tenant_id);
            }
            TenantType::Market => {
                let query = &basic_query.replace("{JOIN_CLAUSE}", "");

                builder = QueryBuilder::new(query);
                builder.push(" AND o.market_id=").push_bind(tenant_id);
            }
            TenantType::Customer => {
                let query = &basic_query.replace("{JOIN_CLAUSE}", "");

                builder = QueryBuilder::new(query);
                builder.push(" AND o.customer_id=").push_bind(tenant_id);
            }
        }

        if let Some(order_status) = query_params.order_status.as_ref() {
            debug!("order_status: {}", order_status);
            builder.push(" AND o.order_status=").push_bind(order_status);
        }

        // Filter by delivery_date if provided
        if let Some(delivery_date_str) = &query_params.delivery_date {
            let delivery_date = NaiveDate::parse_from_str(delivery_date_str, "%Y-%m-%d")
                .map_err(|_| AppError::Validation("Invalid delivery_date format. Use YYYY-MM-DD".to_string()))?;
            builder.push(" AND o.delivery_date=").push_bind(delivery_date);
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
                    rer.delivered_qty      AS "actual_quantity?"
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
                .map(|row| {
                    (
                        row.order_detail_id,
                        row.total_inspected_qty.unwrap_or(Decimal::ZERO),
                    )
                })
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
                inspection_at: inspection_at
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
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
                od.remark,
                od.accepted_qty,
                tiu.temp_url as image_url
            FROM order_details od
            LEFT JOIN temp_image_urls tiu ON od.product_code = tiu.product_code
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
            delivery_date: order_info.delivery_date,
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

    async fn get_provider_return_exchange_orders(
        &self,
        provider_hash: &str,
    ) -> Result<Vec<ProviderReturnExchangeOrderResponse>, AppError> {
        // 获取供应商ID
        let provider_id = self
            .get_tenant_id_by_tenant_hash_and_tenant_type(provider_hash, "PROVIDER")
            .await?;

        // 查询需要退换货的订单信息
        // 从 return_exchange_records 表中获取该供应商需要退换货的订单
        let orders = sqlx::query!(
            r#"
            SELECT DISTINCT
                o.order_code,
                o.order_status,
                o.created_at,
                COUNT(DISTINCT rer.order_detail_id) as total_sku_count
            FROM return_exchange_records rer
            INNER JOIN order_details od ON rer.order_detail_id = od.id
            INNER JOIN orders o ON od.order_id = o.id
            INNER JOIN provider_orders_assignments poa ON o.id = poa.order_id
            WHERE poa.provider_id = ?
                -- AND rer.status = 'PENDING'
                AND o.deleted_at IS NULL
            GROUP BY o.id, o.order_code, o.order_status, o.created_at
            ORDER BY o.created_at DESC
            "#,
            provider_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get provider return exchange orders"))?;

        let mut result = Vec::new();

        for order in orders {
            // 获取每个订单的退换货商品详情
            let items = sqlx::query!(
                r#"
                SELECT DISTINCT
                    od.product_code,
                    od.product_name,
                    od.unit as weight,
                    rer.operation_type,
                    rer.status,
                    rer.delivered_qty,
                    rer.reason,
                    rer.evidence_images
                FROM return_exchange_records rer
                INNER JOIN order_details od ON rer.order_detail_id = od.id
                INNER JOIN orders o ON od.order_id = o.id
                INNER JOIN provider_orders_assignments poa ON o.id = poa.order_id
                WHERE poa.provider_id = ?
                    AND o.order_code = ?
                   -- AND rer.status = 'PENDING'
                    AND o.deleted_at IS NULL
                ORDER BY od.product_code
                "#,
                provider_id,
                order.order_code
            )
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get return exchange items"))?;

            let return_exchange_items: Vec<ProviderReturnExchangeItem> = items
                .into_iter()
                .map(|item| ProviderReturnExchangeItem {
                    product_code: item.product_code,
                    product_name: item.product_name,
                    weight: item.weight,
                    operation_type: item.operation_type,
                    status: item.status,
                    delivered_qty: item.delivered_qty,
                    reason: item.reason,
                    evidence_images: item.evidence_images,
                })
                .collect();

            result.push(ProviderReturnExchangeOrderResponse {
                order_code: order.order_code,
                total_sku_count: order.total_sku_count as i64,
                created_at: order.created_at,
                order_status: order.order_status,
                items: return_exchange_items,
            });
        }

        Ok(result)
    }

    async fn get_market_dashboard_stats(
        &self,
        tenant_hash: &str,
    ) -> Result<MarketOrderStatisticsResponse, AppError> {
        use crate::dto::order::{
            Metadata, PendingAssignmentMetadata, TrendItem, Trends,
        };

        // Get market tenant id
        let market_id = self
            .get_tenant_id_by_tenant_hash_and_tenant_type(tenant_hash, "MARKET")
            .await?;

        let today = Local::now().date_naive();
        let yesterday = today
            .pred_opt()
            .ok_or_else(|| AppError::Internal("无法计算昨天的日期".to_string()))?;

        // Query all order statistics in one query
        let order_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(CASE WHEN DATE(created_at) = ? THEN 1 END) as new_orders_today,
                COUNT(CASE WHEN DATE(created_at) = ? AND order_status = 'PENDING' THEN 1 END) as new_orders_pending_today,
                COUNT(CASE WHEN DATE(created_at) = ? THEN 1 END) as yesterday_total_orders,
                COUNT(CASE WHEN DATE(created_at) = ? AND (order_status = 'EXCHANGE_DELIVERING' OR order_status = 'SUPPLIER_DELIVERING') THEN 1 END) as pending_inspection,
                COUNT(CASE WHEN DATE(created_at) = ? AND order_status = 'COMPLETED' THEN 1 END) as completed_today,
                COUNT(CASE WHEN DATE(created_at) = ? AND order_status = 'COMPLETED' THEN 1 END) as completed_yesterday
            FROM orders
            WHERE market_id = ?
            AND DATE(created_at) IN (?, ?)
            AND deleted_at IS NULL
            "#,
            today,
            today,
            yesterday,
            today,
            today,
            yesterday,
            market_id,
            today,
            yesterday
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get order statistics"))?;

        let new_orders_today = order_stats.new_orders_today;
        let yesterday_total_orders = order_stats.yesterday_total_orders;
        let pending_assignment = order_stats.new_orders_pending_today; // Same as new_orders_today (both are PENDING status)
        let pending_inspection = order_stats.pending_inspection;
        let completed_today = order_stats.completed_today;
        let completed_yesterday = order_stats.completed_yesterday;

        // Calculate newOrders trend
        let new_orders_trend = if yesterday_total_orders > 0 {
            let diff = new_orders_today as f64 - yesterday_total_orders as f64;
            let percentage = (diff / yesterday_total_orders as f64) * 100.0;
            let value = if percentage >= 0.0 {
                format!("+{:.1}%", percentage)
            } else {
                format!("{:.1}%", percentage)
            };
            Some(TrendItem {
                value,
                up: percentage >= 0.0,
            })
        } else {
            Some(TrendItem {
                value: format!("+{}", new_orders_today),
                up: true,
            })
        };

        // Query today's and yesterday's exceptions in one query
        let exceptions_stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT CASE WHEN DATE(rer.created_at) = ? THEN o.id END) as exceptions_today,
                COUNT(DISTINCT CASE WHEN DATE(rer.created_at) = ? THEN o.id END) as exceptions_yesterday
            FROM return_exchange_records rer
            INNER JOIN order_details od ON rer.order_detail_id = od.id
            INNER JOIN orders o ON od.order_id = o.id
            WHERE o.market_id = ?
            AND DATE(rer.created_at) IN (?, ?)
            AND o.deleted_at IS NULL
            "#,
            today,
            yesterday,
            market_id,
            today,
            yesterday
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get exceptions statistics"))?;

        let exceptions_today = exceptions_stats.exceptions_today;
        let exceptions_yesterday = exceptions_stats.exceptions_yesterday;

        // Calculate exceptions trend
        let exceptions_trend = if exceptions_yesterday > 0 {
            let diff = exceptions_today as f64 - exceptions_yesterday as f64;
            let percentage = (diff / exceptions_yesterday as f64) * 100.0;
            let value = if percentage >= 0.0 {
                format!("+{:.1}%", percentage)
            } else {
                format!("{:.1}%", percentage)
            };
            Some(TrendItem {
                value,
                up: percentage >= 0.0,
            })
        } else if exceptions_today > 0 {
            Some(TrendItem {
                value: format!("+{}", exceptions_today),
                up: true,
            })
        } else {
            let diff = exceptions_today - exceptions_yesterday;
            Some(TrendItem {
                value: if diff >= 0 {
                    format!("+{}", diff)
                } else {
                    format!("{}", diff)
                },
                up: diff >= 0,
            })
        };


        // Calculate completed trend
        let completed_trend = if completed_yesterday > 0 {
            let diff = completed_today as f64 - completed_yesterday as f64;
            let percentage = (diff / completed_yesterday as f64) * 100.0;
            let value = if percentage >= 0.0 {
                format!("+{:.1}%", percentage)
            } else {
                format!("{:.1}%", percentage)
            };
            Some(TrendItem {
                value,
                up: percentage >= 0.0,
            })
        } else if completed_today > 0 {
            Some(TrendItem {
                value: format!("+{}", completed_today),
                up: true,
            })
        } else {
            Some(TrendItem {
                value: "+0%".to_string(),
                up: true,
            })
        };

        Ok(MarketOrderStatisticsResponse {
            new_orders: new_orders_today,
            pending_assignment,
            pending_inspection,
            exceptions: exceptions_today,
            completed: completed_today,
            trends: Trends {
                new_orders: new_orders_trend,
                exceptions: exceptions_trend,
                completed: completed_trend,
            },
            metadata: Metadata {
                pending_assignment: PendingAssignmentMetadata {
                    subtitle: "急需处理".to_string(),
                    active: pending_assignment > 0,
                },
            },
        })
    }

    async fn get_provider_dashboard_stats(
        &self,
        provider_hash: &str,
        delivery_date: Option<NaiveDate>,
    ) -> Result<ProviderDashboardStatsDTO, AppError> {
        use sqlx::{MySql, QueryBuilder};
        use tracing::debug;

        debug!(
            "Fetching dashboard stats for provider_hash: {}, delivery_date: {:?}",
            provider_hash, delivery_date
        );

        // 查询待配送订单总数（ASSIGNED 或 SUPPLIER_PREPARING）
        let mut pending_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT COUNT(DISTINCT o.id) as count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            WHERE t.name_hash = 
            "#,
        );

        pending_query.push_bind(provider_hash);
        pending_query.push(" AND t.tenant_type = 'PROVIDER'");
        pending_query.push(" AND t.deleted_at IS NULL");
        pending_query.push(" AND o.order_status IN ('ASSIGNED', 'SUPPLIER_PREPARING')");
        pending_query.push(" AND o.deleted_at IS NULL");

        if let Some(date) = delivery_date {
            pending_query.push(" AND o.delivery_date = ");
            pending_query.push_bind(date);
        }

        let pending_delivery = pending_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get pending delivery orders count"))?;

        // 合并查询：同时统计配送中订单总数（PREPARING）和已完成配送订单总数（DELIVERED）
        let mut delivery_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT 
                COUNT(DISTINCT CASE WHEN pd.delivery_status = 'PREPARING' THEN o.id END) as delivering_count,
                COUNT(DISTINCT CASE WHEN pd.delivery_status = 'DELIVERED' THEN o.id END) as completed_count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            JOIN provider_deliveries pd ON pd.assignment_id = poa.id
            WHERE t.name_hash = 
            "#,
        );

        delivery_query.push_bind(provider_hash);
        delivery_query.push(" AND t.tenant_type = 'PROVIDER'");
        delivery_query.push(" AND t.deleted_at IS NULL");
        delivery_query.push(" AND o.deleted_at IS NULL");
        delivery_query.push(" AND pd.delivery_status IN ('PREPARING', 'DELIVERED')");

        if let Some(date) = delivery_date {
            delivery_query.push(" AND o.delivery_date = ");
            delivery_query.push_bind(date);
        }

        let delivery_stats = delivery_query
            .build_query_as::<DeliveryStatsRow>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get delivery orders count"))?;

        let delivering = delivery_stats.delivering_count;
        let completed = delivery_stats.completed_count;

        // 查询退换货任务总数（return_exchange_records 表中 status = 'PENDING'）
        let mut return_exchange_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT COUNT(DISTINCT rer.id) as count
            FROM return_exchange_records rer
            JOIN order_details od ON rer.order_detail_id = od.id
            JOIN orders o ON od.order_id = o.id
            JOIN provider_orders_assignments poa ON o.id = poa.order_id
            JOIN tenants t ON poa.provider_id = t.id
            WHERE t.name_hash = 
            "#,
        );

        return_exchange_query.push_bind(provider_hash);
        return_exchange_query.push(" AND t.tenant_type = 'PROVIDER'");
        return_exchange_query.push(" AND t.deleted_at IS NULL");
        return_exchange_query.push(" AND rer.status = 'PENDING'");
        return_exchange_query.push(" AND o.deleted_at IS NULL");

        if let Some(date) = delivery_date {
            return_exchange_query.push(" AND o.delivery_date = ");
            return_exchange_query.push_bind(date);
        }

        let return_exchange_tasks = return_exchange_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get return exchange tasks count"))?;

        // 查询待备货 SKU 总数（订单状态为 ASSIGNED 的订单的 product_code 去重）
        let mut pending_stock_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT COUNT(DISTINCT od.product_code) as count
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            JOIN order_details od ON o.id = od.order_id
            WHERE t.name_hash = 
            "#,
        );

        pending_stock_query.push_bind(provider_hash);
        pending_stock_query.push(" AND t.tenant_type = 'PROVIDER'");
        pending_stock_query.push(" AND t.deleted_at IS NULL");
        pending_stock_query.push(" AND o.order_status = 'ASSIGNED'");
        pending_stock_query.push(" AND o.deleted_at IS NULL");

        if let Some(date) = delivery_date {
            pending_stock_query.push(" AND o.delivery_date = ");
            pending_stock_query.push_bind(date);
        }

        let pending_stock_skus = pending_stock_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get pending stock SKUs count"))?;

        // 查询已签收的 SKU 数量（COMPLETED 订单中，CUSTOMER 验收结果为 PASS 或 PARTIAL，且 accepted=1 的 inspected_qty 总和）
        let mut accepted_skus_query = QueryBuilder::<MySql>::new(
            r#"
            SELECT COALESCE(SUM(oii.inspected_qty), 0) as accepted_skus_qty
            FROM provider_orders_assignments poa
            JOIN tenants t ON poa.provider_id = t.id
            JOIN orders o ON poa.order_id = o.id
            JOIN order_inspections oi ON oi.order_id = o.id
            JOIN order_inspection_items oii ON oii.inspection_id = oi.id
            WHERE t.name_hash = 
            "#,
        );

        accepted_skus_query.push_bind(provider_hash);
        accepted_skus_query.push(" AND t.tenant_type = 'PROVIDER'");
        accepted_skus_query.push(" AND t.deleted_at IS NULL");
        accepted_skus_query.push(" AND o.order_status = 'COMPLETED'");
        accepted_skus_query.push(" AND o.deleted_at IS NULL");
        accepted_skus_query.push(" AND oi.inspection_result IN ('PASS', 'PARTIAL')");
        accepted_skus_query.push(" AND oi.inspected_by_type = 'CUSTOMER'");
        accepted_skus_query.push(" AND oii.accepted = 1");

        if let Some(date) = delivery_date {
            accepted_skus_query.push(" AND o.delivery_date = ");
            accepted_skus_query.push_bind(date);
        }

        let accepted_skus_qty = accepted_skus_query
            .build_query_scalar::<sqlx::types::Decimal>()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get accepted SKUs quantity"))?;

        debug!(
            "Dashboard stats - pending_delivery: {}, delivering: {}, completed: {}, return_exchange: {}, pending_stock: {}, accepted_skus_qty: {}",
            pending_delivery, delivering, completed, return_exchange_tasks, pending_stock_skus, accepted_skus_qty
        );

        Ok(ProviderDashboardStatsDTO {
            pending_delivery_orders: pending_delivery,
            delivering_orders: delivering,
            completed_orders: completed,
            return_exchange_tasks: return_exchange_tasks,
            pending_stock_skus: pending_stock_skus,
            accepted_skus_qty: accepted_skus_qty,
        })
    }

    async fn get_customer_dashboard_stats(
        &self,
        customer_hash: &str,
        date: Option<NaiveDate>,
    ) -> Result<DashboardStatsDTO, AppError> {
        // Get customer_id from tenant_hash
        let record = sqlx::query!(
            r#"SELECT t.id AS customer_id 
               FROM tenants t 
               WHERE t.name_hash = ? AND t.tenant_type = 'CUSTOMER' AND t.deleted_at IS NULL"#,
            customer_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get customer information"))?
        .ok_or_else(|| AppError::not_found("Customer not found"))?;

        // Use provided date or current date
        let target_date = date.unwrap_or_else(|| Local::now().date_naive());

        // Single query to get all statistics using COUNT with CASE WHEN
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(CASE WHEN order_status = 'PENDING' THEN 1 END) as pending_confirmation,
                COUNT(CASE WHEN order_status IN ('MARKET_DELIVERING', 'EXCHANGE_NEW_DELIVERING') THEN 1 END) as in_transit,
                COUNT(CASE WHEN order_status = 'COMPLETED' THEN 1 END) as arriving_today,
                COUNT(CASE WHEN order_status = 'CUSTOMER_INSPECTING' THEN 1 END) as in_acceptance,
                COUNT(CASE WHEN order_status IN ('RETURN_REQUESTED', 'EXCHANGE_REQUESTED') THEN 1 END) as exceptions
            FROM orders
            WHERE customer_id = ? 
              AND DATE(created_at) = ?
              AND deleted_at IS NULL
            "#,
            record.customer_id,
            target_date
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get dashboard statistics"))?;

        Ok(DashboardStatsDTO {
            pending_confirmation: stats.pending_confirmation,
            in_transit: stats.in_transit,
            arriving_today: stats.arriving_today,
            in_acceptance: stats.in_acceptance,
            exceptions: stats.exceptions,
        })
    }
}
