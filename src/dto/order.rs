use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Decimal};
use validator::{Validate, ValidationError};

use crate::dto::{validate_delivery_date, validate_phone};


fn validate_delivered_quantity_positive(value: &Decimal) -> Result<(), ValidationError> {
    if value <= &Decimal::ZERO {
        return Err(ValidationError::new("delivered_quantity_must_be_positive"));
    }
    Ok(())
}

/// Data transfer object for creating a new order
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct CreateOrderRequestDTO {
    #[validate(nested)]
    #[serde(rename = "deliveryInfo")]
    pub(crate) delivery_info: DeliveryInfo,

    #[serde(rename = "items")]
    pub(crate) items: Vec<OrderedItem>,
}

/// Delivery information for an order
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct DeliveryInfo {
    #[serde(rename = "deliveryDate")]
    #[validate(custom(function = "validate_delivery_date"))]
    pub(crate) delivery_date: String,

    #[validate(length(min = 4, max = 255))]
    #[serde(rename = "address")]
    pub(crate) delivery_address: String,

    #[validate(length(min = 2, max = 32))]
    #[serde(rename = "contactName")]
    pub(crate) contact_name: String,

    #[validate(custom(function = validate_phone))]
    #[serde(rename = "contactPhone")]
    pub(crate) contact_phone: String,
}

/// Individual item in an order creation request
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct OrderedItem {

    #[serde(rename = "categoryId")]
    pub(crate) category_level_1_id: i32,

    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "orderedQty")]
    pub(crate) ordered_qty: Decimal,

    pub(crate) remark: Option<String>,

    #[serde(rename = "processingServices")]
    pub(crate) processing_services: Vec<ProcessingService>,
}

/// Processing service for an order item
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ProcessingService {
    #[serde(rename = "type")]
    pub(crate) name: String,

    pub(crate) description: Option<String>,
}

/// Response DTO for order operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub(crate) struct OrderResponse {
    #[serde(rename = "orderCode")]
    pub(crate) order_code: String,

    #[serde(rename = "totalAmount")]
    pub(crate) ordered_amount: Decimal,

    #[serde(rename = "actualAmount")]
    pub(crate) net_amount: Decimal,

    #[serde(rename = "deliveryDate")]
    pub(crate) delivery_date: NaiveDate,

    #[serde(rename = "deliveryAddress")]
    pub(crate) delivery_address: String,

    #[serde(rename = "orderStatus")]
    pub(crate) order_status: String,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: Option<DateTime<Local>>,

    #[serde(rename = "marketName")]
    pub(crate) market_name: String,

    #[serde(rename = "marketAddress")]
    pub(crate) market_address: String,

    #[serde(rename = "customerName")]
    pub(crate) customer_name: String,
    
    #[serde(rename = "customerAddress")]
    pub(crate) customer_address: String,

    #[serde(rename = "contactName")]
    pub(crate) contact_name: String,

    #[serde(rename = "contactPhone")]
    pub(crate) contact_phone: String,

    #[serde(rename = "marketContactNumber")]
    pub(crate) market_contact_number: Option<String>,

    #[serde(rename = "marketContactorName")]
    pub(crate) market_contactor_name: Option<String>,

    #[serde(rename = "shipperName")]
    pub(crate) shipper_name: Option<String>,

    #[serde(rename = "shipperPhone")]
    pub(crate) shipper_phone: Option<String>,

    #[serde(rename = "skuCount")]
    pub(crate) sku_count: Option<i64>,

    #[serde(rename = "urgent")]
    pub(crate) urgent: bool,
}

/// Query parameters for order listing
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub(crate) struct OrderQueryParams {
    #[serde(rename = "page")]
    pub(crate) page: Option<i32>,

    #[serde(rename = "pageSize")]
    pub(crate) page_size: Option<i32>,

    #[serde(rename = "orderStatus")]
    pub(crate) order_status: Option<String>,

    /// 配送日期：YYYY-MM-DD，不传则不按日期筛选
    #[serde(rename = "deliveryDate")]
    pub(crate) delivery_date: Option<String>,
}

/// 订单验收项信息
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderInspectionItem {
    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i32,

    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "categoryName")]
    pub(crate) category_name: String,

    #[serde(rename = "result")]
    pub(crate) result: String,

    #[serde(rename = "needToInspection")]
    pub(crate) need_to_inspection: Decimal,

    #[serde(rename = "inspectedQty", skip_serializing_if = "Option::is_none")]
    pub(crate) inspected_qty: Option<Decimal>,

    #[serde(rename = "quantity", skip_serializing_if = "Option::is_none")]
    pub(crate) quantity: Option<Decimal>,

    #[serde(rename = "remark", skip_serializing_if = "Option::is_none")]
    pub(crate) remark: Option<String>,
}

/// 订单验收信息
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderInspection {
    #[serde(rename = "inspectionId")]
    pub(crate) inspection_id: i32,

    #[serde(rename = "parent_id", skip_serializing_if = "Option::is_none")]
    pub(crate) parent_id: Option<i32>,

    #[serde(rename = "inspectionRound")]
    pub(crate) inspection_round: i32,

    #[serde(rename = "inspectedByType")]
    pub(crate) inspected_by_type: String,

    #[serde(rename = "inspectedById")]
    pub(crate) inspected_by_id: i32,

    #[serde(rename = "result")]
    pub(crate) result: String,

    #[serde(rename = "inspectedAt", skip_serializing_if = "Option::is_none")]
    pub(crate) inspected_at: Option<DateTime<Utc>>,

    #[serde(rename = "items")]
    pub(crate) items: Vec<OrderInspectionItem>,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct OrderDetail {
    pub(crate) id: i32,

    #[serde(rename = "productId")]
    pub(crate) product_code: String,

    #[serde(rename = "name")]
    pub(crate) product_name: String,

    #[serde(rename = "categoryId")]
    pub(crate) category_id: i32,

    #[serde(rename = "category")]
    pub(crate) category_name: String,

    #[serde(rename = "imageUrl")]
    pub(crate) image_url: Option<String>,

    pub(crate) unit: String,
    #[serde(rename = "orderedQty")]
    pub(crate) ordered_qty: Decimal,

    #[serde(rename = "acceptedQty")]
    pub(crate) accepted_qty: Option<Decimal>,

    #[serde(rename = "unitPrice")]
    pub(crate) unit_price: Decimal,

    #[serde(rename = "discountRate")]
    pub(crate) discount_rate: Decimal,

    #[serde(rename = "discountedUnitPrice")]
    pub(crate) discounted_unit_price: Decimal,

    #[serde(rename = "netAmount")]
    pub(crate) net_amount: Option<Decimal>,

    #[serde(rename = "orderedAmount")]
    pub(crate) ordered_amount: Decimal,

    #[serde(rename = "processingRequirements")]
    pub(crate) processing_requirements: Option<String>,

    pub(crate) remark: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct ExchangeAndReturnOrderDetailResponse {
    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i32,

    #[serde(rename = "operationType")]
    pub(crate) operation_type: String,

    pub(crate) quantity: Decimal,
    pub(crate) reason: String,
    pub(crate) unit: String,
    pub(crate) status: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "evidenceImages")]
    pub(crate) evidence_images: Option<String>,

    #[serde(rename = "processedBy")]
    pub(crate) processed_by: Option<String>,

    #[serde(rename = "processedAt")]
    pub(crate) processed_at: Option<DateTime<Utc>>,

    #[serde(rename = "actualQuantity")]
    pub(crate) actual_quantity: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow, Validate)]
pub(crate) struct DeliverQuantity {
    pub(crate) id: i32,

    #[serde(rename = "deliveredQuantity")]
    #[validate(custom(function = "validate_delivered_quantity_positive"))]
    pub(crate) delivered_quantity: Decimal,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct DeliverToMarketDTO {
    // #[serde(rename = "stockedBy")]
    // pub(crate) stocked_by: String,

    #[validate(nested)]
    pub(crate) items: Vec<DeliverQuantity>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct ExchangeDTO {
    pub(crate) items: Vec<ExchangeItem>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ExchangeItem {
    // order detail id
    #[serde(rename = "orderDetailId")]
    pub(crate) id: i32,

    #[serde(rename = "actualQuantity")]
    pub(crate) actual_quantity: Decimal,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct AcceptedOrderResponseDTO {
    #[serde(rename = "orderCode")]
    pub(crate) order_code: String,

    #[serde(rename = "orderStatus")]
    pub(crate) order_status: String,

    #[serde(rename = "acceptedAt")]
    pub(crate) accepted_at: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct OrderReceiptDTO {
    pub(crate) receipt: OrderReceipt,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct ProductsSummaryWithOrdersDTO {
    #[serde(rename = "productId")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "totalQuantity")]
    pub(crate) total_quantity: Option<Decimal>,

    #[serde(rename = "unit")]
    pub(crate) unit: String,

    #[serde(rename = "processingRequirements")]
    pub(crate) processing_requirements: Option<String>,

    #[serde(rename = "customerName")]
    pub(crate) customer_name: Option<String>,

    pub(crate) remark: Option<String>,
}

/// 供应商 Dashboard 统计数据
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ProviderDashboardStatsDTO {
    #[serde(rename = "pendingDeliveryOrders")]
    pub(crate) pending_delivery_orders: i64,

    #[serde(rename = "deliveringOrders")]
    pub(crate) delivering_orders: i64,

    #[serde(rename = "completedOrders")]
    pub(crate) completed_orders: i64,

    #[serde(rename = "returnExchangeTasks")]
    pub(crate) return_exchange_tasks: i64,

    #[serde(rename = "pendingStockSkus")]
    pub(crate) pending_stock_skus: i64,

    #[serde(rename = "acceptedSkusQty")]
    pub(crate) accepted_skus_qty: Decimal,
}

/// 供应商今天已交付的商品聚合数据
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct ProviderTodayDeliveredProductsDTO {
    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i32,

    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "categoryName")]
    pub(crate) category_name: Option<String>,

    #[serde(rename = "unit")]
    pub(crate) unit: String,

    #[serde(rename = "totalActualQty")]
    pub(crate) total_actual_qty: Decimal,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderReceipt {
    #[serde(rename = "id")]
    pub(crate) order_inspection_id: i32,

    #[serde(rename = "orderId")]
    pub(crate) order_code: String,

    #[serde(rename = "productId")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "operationType")]
    pub(crate) operation_type: ReceiptOperationType,

    pub(crate) quantity: Decimal,

    pub(crate) reason: String,

    pub(crate) unit: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "evidenceImages")]
    pub(crate) evidence_images: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow, Validate)]
pub(crate) struct DispatchOrderDTO {
    #[serde(rename = "providerId")]
    pub(crate) provider_id: i32,

    #[serde(rename = "confirmedBy")]
    pub(crate) confirmed_by: String,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct ExchangeItemQuantityUpdateRequest {
    #[serde(rename = "actualQuantity")]
    pub(crate) actual_quantity: Decimal,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) enum ReceiptOperationType {
    #[serde(rename = "SIGN")]
    Sign, // Receipt confirmation

    #[serde(rename = "RETURN")]
    Return, // Return goods

    #[serde(rename = "EXCHANGE")]
    Exchange, // Exchange goods
}

impl ReceiptOperationType {
    pub(crate) fn to_str(self) -> &'static str {
        match self {
            ReceiptOperationType::Sign => "SIGN",
            ReceiptOperationType::Return => "RETURN",
            ReceiptOperationType::Exchange => "EXCHANGE",
        }
    }
}

impl TryFrom<String> for ReceiptOperationType {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "SIGN" => Ok(ReceiptOperationType::Sign),
            "RETURN" => Ok(ReceiptOperationType::Return),
            "EXCHANGE" => Ok(ReceiptOperationType::Exchange),
            _ => Err(format!("无效的收据操作类型: {}", s)),
        }
    }
}

impl From<ReceiptOperationType> for String {
    fn from(operation_type: ReceiptOperationType) -> Self {
        match operation_type {
            ReceiptOperationType::Sign => "SIGN".to_string(),
            ReceiptOperationType::Return => "RETURN".to_string(),
            ReceiptOperationType::Exchange => "EXCHANGE".to_string(),
        }
    }
}

/// 供应商订单轮次响应
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ProviderOrderRound {
    #[serde(rename = "deliveryStatus")]
    pub(crate) delivery_status: String,

    #[serde(rename = "deliveryType")]
    pub(crate) delivery_type: String,

    pub(crate) items: Vec<ProviderOrderItem>,
}

/// 供应商订单项响应
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ProviderOrderItem {
    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i32,

    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "categoryId")]
    pub(crate) category_id: i32,

    #[serde(rename = "categoryName")]
    pub(crate) category_name: String,

    #[serde(rename = "needToDeliverQty")]
    pub(crate) need_to_deliver_qty: Decimal,

    #[serde(rename = "actualQty")]
    pub(crate) actual_qty: Option<Decimal>,

    #[serde(rename = "unitPrice")]
    pub(crate) unit_price: Decimal,

    #[serde(rename = "imageUrl")]
    pub(crate) image_url: Option<String>,

    pub(crate) unit: String,

    #[serde(rename = "processingRequirements")]
    pub(crate) processing_requirements: Option<String>,

    pub(crate) remark: Option<ProviderOrderRemark>,
}

/// 供应商订单备注
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ProviderOrderRemark {
    #[serde(rename = "evidenceImages")]
    pub(crate) evidence_images: Option<String>,

    pub(crate) reason: Option<String>,
}

/// 供应商订单响应
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ProviderOrderResponse {
    #[serde(rename = "orderCode")]
    pub(crate) order_code: String,

    #[serde(rename = "orderStatus")]
    pub(crate) order_status: String,

    #[serde(rename = "deliveryDate")]
    pub(crate) delivery_date: NaiveDate,

    #[serde(rename = "receiverName")]
    pub(crate) receiver_name: Option<String>,

    #[serde(rename = "customerName")]
    pub(crate) customer_name: String,

    #[serde(rename = "orderedAmount")]
    pub(crate) ordered_amount: Decimal,

    #[serde(rename = "discountAmount")]
    pub(crate) discount_amount: Decimal,

    #[serde(rename = "netAmount")]
    pub(crate) net_amount: Decimal,

    #[serde(rename = "receiverPhone")]
    pub(crate) receiver_phone: Option<String>,

    #[serde(rename = "deliveryAddress")]
    pub(crate) delivery_address: String,

    #[serde(rename = "shipperName")]
    pub(crate) shipper_name: Option<String>,

    #[serde(rename = "shipperPhone")]
    pub(crate) shipper_phone: Option<String>,

    pub(crate) current: ProviderOrderRound,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,

    #[serde(rename = "marketContactNumber")]
    pub(crate) market_contact_number: Option<String>,

    #[serde(rename = "marketContactorName")]
    pub(crate) market_contactor_name: Option<String>,
}

/// MARKET 用户订单详情响应
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct MarketOrderDetailResponse {
    #[serde(rename = "orderCode")]
    pub(crate) order_code: String,

    #[serde(rename = "orderStatus")]
    pub(crate) order_status: String,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,

    #[serde(rename = "customerName")]
    pub(crate) customer_name: Option<String>,

    #[serde(rename = "deliveryAddress")]
    pub(crate) delivery_address: Option<String>,

    #[serde(rename = "deliveryDate")]
    pub(crate) delivery_date: NaiveDate,

    #[serde(rename = "discountAmount")]
    pub(crate) discount_amount: Decimal,

    #[serde(rename = "netAmount")]
    pub(crate) net_amount: Decimal,

    #[serde(rename = "orderedAmount")]
    pub(crate) ordered_amount: Decimal,

    #[serde(rename = "receiverName")]
    pub(crate) receiver_name: Option<String>,

    #[serde(rename = "receiverPhone")]
    pub(crate) receiver_phone: Option<String>,

    #[serde(rename = "shipperName")]
    pub(crate) shipper_name: Option<String>,

    #[serde(rename = "shipperPhone")]
    pub(crate) shipper_phone: Option<String>,

    pub(crate) details: Vec<OrderDetail>,

    pub(crate) rounds: Vec<MarketOrderRound>,
}

/// MARKET 订单轮次
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct MarketOrderRound {
    pub(crate) round: i32,

    #[serde(rename = "deliveryType")]
    pub(crate) delivery_type: String,

    #[serde(rename = "deliveredAt")]
    pub(crate) delivered_at: Option<DateTime<Utc>>,

    #[serde(rename = "inspectionResult")]
    pub(crate) inspection_result: String,

    #[serde(rename = "inspectionAt")]
    pub(crate) inspection_at: Option<DateTime<Utc>>,

    pub(crate) items: Vec<MarketOrderItem>,
}

/// MARKET 订单商品项
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct MarketOrderItem {
    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i32,

    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "categoryId")]
    pub(crate) category_id: i32,

    #[serde(rename = "categoryName")]
    pub(crate) category_name: String,

    pub(crate) unit: String,

    #[serde(rename = "imageUrl")]
    pub(crate) image_url: Option<String>,

    #[serde(rename = "unitPrice")]
    pub(crate) unit_price: Decimal,

    #[serde(rename = "orderedQty")]
    pub(crate) ordered_qty: Decimal,

    #[serde(rename = "needToInspectQty")]
    pub(crate) need_to_inspect_qty: Decimal,

    #[serde(rename = "acceptedQty")]
    pub(crate) accepted_qty: Option<Decimal>,

    #[serde(rename = "exchangeQty")]
    pub(crate) exchange_qty: Option<Decimal>,

    #[serde(rename = "returnQty")]
    pub(crate) return_qty: Option<Decimal>,

    #[serde(rename = "inspectionStatus")]
    pub(crate) inspection_status: String,

    #[serde(rename = "processingRequirements")]
    pub(crate) processing_requirements: Option<String>,

    pub(crate) remark: Option<ProviderOrderRemark>,
}

/// CUSTOMER 订单详情响应
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct CustomerOrderDetailResponse {
    #[serde(rename = "orderCode")]
    pub(crate) order_code: String,

    #[serde(rename = "orderStatus")]
    pub(crate) order_status: String,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,

    #[serde(rename = "customerName")]
    pub(crate) customer_name: Option<String>,

    #[serde(rename = "deliveryAddress")]
    pub(crate) delivery_address: Option<String>,

    #[serde(rename = "deliveryDate")]
    pub(crate) delivery_date: NaiveDate,

    #[serde(rename = "discountAmount")]
    pub(crate) discount_amount: Decimal,

    #[serde(rename = "netAmount")]
    pub(crate) net_amount: Decimal,

    #[serde(rename = "orderedAmount")]
    pub(crate) ordered_amount: Decimal,

    #[serde(rename = "receiverName")]
    pub(crate) receiver_name: Option<String>,

    #[serde(rename = "receiverPhone")]
    pub(crate) receiver_phone: Option<String>,

    #[serde(rename = "shipperName")]
    pub(crate) shipper_name: Option<String>,

    #[serde(rename = "shipperPhone")]
    pub(crate) shipper_phone: Option<String>,

    pub(crate) details: Vec<OrderDetail>,

    pub(crate) rounds: Vec<CustomerOrderRound>,
}

/// CUSTOMER 订单轮次
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct CustomerOrderRound {
    pub(crate) round: i32,

    #[serde(rename = "inspectionResult")]
    pub(crate) inspection_result: String,

    #[serde(rename = "inspectionAt")]
    pub(crate) inspection_at: Option<DateTime<Utc>>,

    pub(crate) items: Vec<CustomerOrderItem>,
}

/// CUSTOMER 订单商品项
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct CustomerOrderItem {
    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i32,

    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "categoryId")]
    pub(crate) category_id: i32,

    #[serde(rename = "categoryName")]
    pub(crate) category_name: String,

    pub(crate) unit: String,

    #[serde(rename = "unitPrice")]
    pub(crate) unit_price: Decimal,

    #[serde(rename = "orderedQty")]
    pub(crate) ordered_qty: Decimal,

    #[serde(rename = "needToInspectQty")]
    pub(crate) need_to_inspect_qty: Option<Decimal>,

    #[serde(rename = "acceptedQty")]
    pub(crate) accepted_qty: Option<Decimal>,

    #[serde(rename = "exchangeQty")]
    pub(crate) exchange_qty: Option<Decimal>,

    #[serde(rename = "returnQty")]
    pub(crate) return_qty: Option<Decimal>,

    #[serde(rename = "inspectionStatus")]
    pub(crate) inspection_status: String,

    #[serde(rename = "processingRequirements")]
    pub(crate) processing_requirements: Option<String>,

    pub(crate) remark: Option<ProviderOrderRemark>,
}

/// 订单交付历史响应
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderDeliveryHistoryResponse {
    #[serde(rename = "orderCode")]
    pub(crate) order_code: String,

    pub(crate) history: Vec<OrderDeliveryHistoryItem>,
}

/// 订单交付历史项
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderDeliveryHistoryItem {
    pub(crate) round: i32,

    #[serde(rename = "deliveryType")]
    pub(crate) delivery_type: String,

    #[serde(rename = "deliveryStatus")]
    pub(crate) delivery_status: String,

    #[serde(rename = "inspectionStatus")]
    pub(crate) inspection_status: String,

    #[serde(rename = "deliveredAt")]
    pub(crate) delivered_at: DateTime<Utc>,

    #[serde(rename = "inspectedAt")]
    pub(crate) inspected_at: Option<DateTime<Utc>>,

    #[serde(rename = "deliveryStaff")]
    pub(crate) delivery_staff: Option<DeliveryStaffInfo>,

    pub(crate) items: Vec<OrderDeliveryHistoryItemDetail>,
}

/// 交付人员信息
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct DeliveryStaffInfo {
    pub(crate) id: String,

    pub(crate) name: String,

    pub(crate) phone: String,
}

/// 订单交付历史商品详情
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderDeliveryHistoryItemDetail {
    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i32,

    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "categoryId")]
    pub(crate) category_id: i32,

    #[serde(rename = "categoryName")]
    pub(crate) category_name: String,

    pub(crate) unit: String,

    #[serde(rename = "unitPrice")]
    pub(crate) unit_price: Decimal,

    #[serde(rename = "orderedQty")]
    pub(crate) ordered_qty: Decimal,

    #[serde(rename = "needToDeliverQty")]
    pub(crate) need_to_deliver_qty: Decimal,

    #[serde(rename = "actualQty")]
    pub(crate) actual_qty: Option<Decimal>,

    #[serde(rename = "inspectedQty")]
    pub(crate) inspected_qty: Option<Decimal>,

    #[serde(rename = "lastInspectionResult")]
    pub(crate) last_inspection_result: Option<String>,

    #[serde(rename = "inspectionAt")]
    pub(crate) inspection_at: Option<DateTime<Utc>>,

    pub(crate) remark: Option<String>,
}

/// 订单检查商品项
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct NeedToInspectionItem {
    #[serde(rename = "inspectionItemId")]
    pub(crate) inspection_item_id: u64,

    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "categoryId")]
    pub(crate) category_id: i32,

    #[serde(rename = "categoryName")]
    pub(crate) category_name: String,

    pub(crate) unit: String,

    #[serde(rename = "unitPrice")]
    pub(crate) unit_price: Decimal,

    #[serde(rename = "orderedQty")]
    pub(crate) ordered_qty: Decimal,

    #[serde(rename = "needToInspectQty")]
    pub(crate) need_to_inspect_qty: Decimal,

    #[serde(rename = "inspectedQty")]
    pub(crate) inspected_qty: Option<Decimal>,

    #[serde(rename = "inspectionStatus")]
    pub(crate) inspection_status: String,

    #[serde(rename = "processingRequirements")]
    pub(crate) processing_requirements: Option<String>,

    #[serde(rename = "discountRate")]
    pub(crate) discount_rate: Decimal,
}

/// 订单检查信息响应
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct NeedToInspection {
    pub(crate) round: i32,

    #[serde(rename = "deliveryType")]
    pub(crate) delivery_type: String,

    #[serde(rename = "deliveredAt")]
    pub(crate) delivered_at: Option<DateTime<Utc>>,

    #[serde(rename = "inspectionAt")]
    pub(crate) inspection_at: Option<DateTime<Utc>>,

    #[serde(rename = "inspectionResult")]
    pub(crate) inspection_result: String,

    #[serde(rename = "deliveryStaff")]
    pub(crate) delivery_staff: Option<DeliveryStaffInfo>,

    pub(crate) items: Vec<NeedToInspectionItem>,
}

/// 供应商退换货订单信息响应
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ProviderReturnExchangeOrderResponse {
    #[serde(rename = "orderCode")]
    pub(crate) order_code: String,

    #[serde(rename = "totalSkuCount")]
    pub(crate) total_sku_count: i64,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,

    #[serde(rename = "orderStatus")]
    pub(crate) order_status: String,

    #[serde(rename = "items")]
    pub(crate) items: Vec<ProviderReturnExchangeItem>,
}

/// 供应商退换货商品项
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ProviderReturnExchangeItem {
    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "weight")]
    pub(crate) weight: String,

    #[serde(rename = "operationType")]
    pub(crate) operation_type: String,

    #[serde(rename = "status")]
    pub(crate) status: String,

    #[serde(rename = "deliveredQty")]
    pub(crate) delivered_qty: Decimal,

    pub(crate) reason: String,

    #[serde(rename = "evidenceImages")]
    pub(crate) evidence_images: Option<String>,
}

/// Market order statistics response DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MarketOrderStatisticsResponse {
    #[serde(rename = "newOrders")]
    pub(crate) new_orders: i64,

    #[serde(rename = "pendingAssignment")]
    pub(crate) pending_assignment: i64,

    #[serde(rename = "pendingInspection")]
    pub(crate) pending_inspection: i64,

    pub(crate) exceptions: i64,

    pub(crate) completed: i64,

    pub(crate) trends: Trends,

    pub(crate) metadata: Metadata,
}

/// Trends information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Trends {
    #[serde(rename = "newOrders")]
    pub(crate) new_orders: Option<TrendItem>,

    pub(crate) exceptions: Option<TrendItem>,

    pub(crate) completed: Option<TrendItem>,
}

/// Trend item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct TrendItem {
    pub(crate) value: String,
    pub(crate) up: bool,
}

/// Metadata information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Metadata {
    #[serde(rename = "pendingAssignment")]
    pub(crate) pending_assignment: PendingAssignmentMetadata,
}

/// Pending assignment metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PendingAssignmentMetadata {
    pub(crate) subtitle: String,
    pub(crate) active: bool,
}

/// Dashboard statistics DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct DashboardStatsDTO {
    #[serde(rename = "pendingConfirmation")]
    pub(crate) pending_confirmation: i64,
    #[serde(rename = "inTransit")]
    pub(crate) in_transit: i64,
    #[serde(rename = "arrivingToday")]
    pub(crate) arriving_today: i64,
    #[serde(rename = "inAcceptance")]
    pub(crate) in_acceptance: i64,
    pub(crate) exceptions: i64,
}

/// Dashboard query parameters
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct DashboardQueryParams {
    /// 日期：YYYY-MM-DD，不传默认今天
    pub(crate) date: Option<String>,
}
