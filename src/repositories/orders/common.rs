use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::order::{
        CustomerOrderDetailResponse, CustomerOrderItem, CustomerOrderRound,
        ExchangeAndReturnOrderDetailResponse, MarketOrderDetailResponse, MarketOrderItem,
        MarketOrderRound, OrderDetail, 
        OrderQueryParams, OrderResponse, ProviderOrderItem,
        ProviderOrderRemark, ProviderOrderResponse, ProviderOrderRound
    },
    map_db_err,
    models::{order_status::OrderStatus, tenant_type::TenantType},
};
use async_trait::async_trait;
use sqlx::{MySql, QueryBuilder};
use tracing::debug;

use chrono::{DateTime, Utc};

use rust_decimal::Decimal;


#[async_trait]
pub(crate) trait CommonOrderRepository: Send + Sync {
    async fn get_orders_by_tenant(
        &self,
        tenant_hash: &str,
        tenant_type: &str,
        query_params: &OrderQueryParams,
    ) -> Result<Vec<OrderResponse>, AppError>;

    // async fn order_by_order_code(
    //     &self,
    //     order_code: &str,
    //     claims: &Claims,
    // ) -> Result<Option<OrderDetailResponse>, AppError>;

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
            SELECT o.id as order_id, o.order_code, o.order_status, o.delivery_date, o.delivery_address, 
                   o.confirmed_by as receiver_name, o.market_contact_number as receiver_phone, o.created_at,
                   mt.name as customer_name, o.ordered_amount, o.discount_amount, o.net_amount,
                   ds.name as shipper_name, ds.phone as shipper_phone
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

        let (order_details, delivery_type): (Vec<ProviderOrderItem>, String) = match OrderStatus::try_from(order_info.order_status.as_str())? {
          OrderStatus::Assigned => {
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
                od.processing_requirements
            FROM order_details od
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
          })
          .collect();
          (items, "NORMAL".to_string())
        }
        OrderStatus::ExchangeRequested => {
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
              od.processing_requirements
            FROM return_exchange_records rer
            INNER JOIN order_details od ON rer.order_detail_id = od.id
            WHERE rer.order_detail_id IN (SELECT order_detail_id FROM provider_deliveries WHERE order_id = ?)
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
          })
          .collect();
          (items, "EXCHANGE".to_string())
        }
          _ => {
            // 获取最新一轮的发货信息
            debug!("get latest delivery------------------");
            let latest_delivery = sqlx::query!(
              r#"
              SELECT pd.id as delivery_id, pd.delivery_status, pd.delivery_type, pd.delivery_round
              FROM provider_deliveries pd
              INNER JOIN provider_orders_assignments poa ON pd.assignment_id = poa.id
              WHERE poa.order_id = ?
              ORDER BY pd.delivery_round DESC
              LIMIT 1
              "#,
              order_info.order_id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get latest delivery"))?;

            match latest_delivery {
              Some(delivery) => {
                let delivery_type = delivery.delivery_type.clone();
                debug!("delivery_type: {}", delivery_type);
                if delivery.delivery_type == "NORMAL" {
                  // NORMAL类型：need_to_deliver_qty = ordered_qty, actual_qty = provider_delivery_items.actual_qty
                  let items = sqlx::query!(
                    r#"
                    SELECT
                       od.id as order_detail_id,
                       od.product_code,
                       od.product_name,
                       od.category_id,
                       od.category_name,
                       od.unit,
                       od.unit_price,
                       od.processing_requirements,
                       od.ordered_qty as need_to_deliver_qty,
                       pdi.actual_qty as actual_qty
                    FROM provider_delivery_items pdi
                    INNER JOIN order_details od ON pdi.order_detail_id = od.id
                    WHERE pdi.delivery_id = ?
                    "#,
                    delivery.delivery_id
                  )
                  .fetch_all(&self.pool)
                  .await
                  .map_err(map_db_err!("Failed to get normal delivery items"))?
                  .into_iter()
                  .map(|row| ProviderOrderItem {
                    order_detail_id: row.order_detail_id,
                    product_code: row.product_code,
                    product_name: row.product_name,
                    category_id: row.category_id,
                    category_name: row.category_name,
                    need_to_deliver_qty: row.need_to_deliver_qty,
                    actual_qty: Some(row.actual_qty),
                    unit_price: row.unit_price,
                    unit: row.unit,
                    processing_requirements: row.processing_requirements,
                  })
                  .collect();
                  (items, delivery_type)
                } else {
                  // EXCHANGE类型：need_to_deliver_qty = return_exchange_records.quantity
                  let items = sqlx::query!(
                    r#"
                    SELECT
                       od.id as order_detail_id,
                       od.product_code,
                       od.product_name,
                       od.category_id,
                       od.category_name,
                       od.unit,
                       od.unit_price,
                       od.processing_requirements,
                       rer.quantity as need_to_deliver_qty,
                       pdi.actual_qty as actual_qty
                    FROM provider_delivery_items pdi
                    INNER JOIN order_details od ON pdi.order_detail_id = od.id
                    LEFT JOIN return_exchange_records rer ON rer.order_detail_id = od.id
                      AND rer.inspection_id IN (
                        SELECT oi.id FROM order_inspections oi
                        WHERE oi.order_id = ?
                        AND oi.inspection_round = (
                          SELECT MAX(inspection_round) FROM order_inspections
                          WHERE order_id = ?
                        )
                       
                      )
                      AND rer.operation_type = 'EXCHANGE'
                    WHERE pdi.delivery_id = ?
                      AND rer.quantity IS NOT NULL
                    "#,
                    order_info.order_id,
                    order_info.order_id,
                    delivery.delivery_id
                  )
                  .fetch_all(&self.pool)
                  .await
                  .map_err(map_db_err!("Failed to get exchange delivery items"))?
                  .into_iter()
                  .map(|row| ProviderOrderItem {
                    order_detail_id: row.order_detail_id,
                    product_code: row.product_code,
                    product_name: row.product_name,
                    category_id: row.category_id,
                    category_name: row.category_name,
                    need_to_deliver_qty: row.need_to_deliver_qty.unwrap_or_default(),
                    actual_qty: Some(row.actual_qty),
                    unit_price: row.unit_price,
                    unit: row.unit,
                    processing_requirements: row.processing_requirements,
                  })
                  .collect();
                  (items, delivery_type)
                }
              }
              None => (Vec::new(), "NORMAL".to_string()),
            }
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
        .map_err(map_db_err!("Failed to get order basic info"))?;

        let order_info = match order_info {
            Some(info) => info,
            None => return Ok(None),
        };

        // 获取所有轮次的发货信息，只关注配送数据
        let delivery_rounds = sqlx::query!(
            r#"
            SELECT pd.delivery_round, pd.delivery_type, pd.delivery_status,
                   pd.delivered_at, pd.id as delivery_id
            FROM provider_deliveries pd
            INNER JOIN provider_orders_assignments poa ON pd.assignment_id = poa.id
            INNER JOIN orders o ON poa.order_id = o.id
            WHERE o.order_code = ?
            ORDER BY pd.delivery_round ASC
            "#,
            order_code
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err!("Failed to get delivery rounds"))?;

        let mut rounds_response = Vec::new();

        for delivery_info in delivery_rounds {
            // 获取该轮次的验收信息（如果有的话）
            let inspection_info = sqlx::query!(
                r#"
                SELECT oi.inspection_result, oi.inspected_at
                FROM order_inspections oi
                WHERE oi.order_id = ? AND oi.inspection_round = ? AND oi.inspected_by_type = 'MARKET'
                "#,
                order_info.order_id,
                delivery_info.delivery_round
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err!("Failed to get inspection info"))?;

            // 获取该轮配送的所有商品
            let round_items = sqlx::query!(
                r#"
                SELECT od.id as order_detail_id, od.product_code, od.product_name,
                       od.category_id, od.category_name, od.unit, od.discounted_unit_price as unit_price,
                       od.ordered_qty, pdi.actual_qty as need_to_inspect_qty,
                       oii.inspected_qty as accepted_qty,
                       CASE
                           WHEN oii.result = 'SIGN' THEN oii.inspected_qty
                           ELSE 0
                       END as accepted_qty_calc,
                       CASE
                           WHEN oii.result = 'EXCHANGE' THEN oii.inspected_qty
                           ELSE NULL
                       END as exchange_qty,
                       CASE
                           WHEN oii.result = 'RETURN' THEN oii.inspected_qty
                           ELSE NULL
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
                FROM provider_delivery_items pdi
                INNER JOIN order_details od ON pdi.order_detail_id = od.id
                INNER JOIN orders o ON od.order_id = o.id
                LEFT JOIN order_inspection_items oii ON od.id = oii.order_detail_id
                    AND oii.inspection_id IN (
                        SELECT oi.id FROM order_inspections oi
                        WHERE oi.order_id = o.id AND oi.inspection_round = ?
                        AND oi.inspected_by_type = 'MARKET'
                    )
                WHERE pdi.delivery_id = ?
                "#,
                delivery_info.delivery_round,
                delivery_info.delivery_id
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

            let (inspection_result, inspection_at) = inspection_info
                .map(|info| (info.inspection_result, info.inspected_at))
                .unwrap_or_else(|| ("PENDING".to_string(), None));

            let market_round = MarketOrderRound {
                round: delivery_info.delivery_round,
                delivery_type: delivery_info.delivery_type,
                delivered_at: delivery_info.delivered_at.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
                inspection_result,
                inspection_at: inspection_at.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
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
                   mt.name as market_name,
                   mt.address as market_address,
                   pt.name as customer_name,
                   pt.address as customer_address,
                   o.contact_name,
                   o.contact_phone,
                   o.market_contact_number,
                   o.confirmed_by as market_contactor_name
                   FROM orders o
                   INNER JOIN tenants mt ON o.market_id = mt.id
                   INNER JOIN tenants pt ON o.customer_id = pt.id
                   {JOIN_CLAUSE} WHERE 1=1 "#;

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
