use crate::repositories::my_sql_repository::MySqlRepository;
use crate::{
    common::AppError,
    dto::order::{
        ExchangeAndReturnOrderDetailResponse, OrderDetail, OrderDetailResponse,
        OrderDelivery, OrderDeliveryItem, OrderQueryParams, OrderResponse, OrderStatusHistory, ReceiptItem, ReceiptOperationType, ReceiptResponse,
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
}
