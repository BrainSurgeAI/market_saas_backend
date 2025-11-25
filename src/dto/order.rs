use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Decimal};
use validator::{Validate, ValidationError};

use crate::dto::{validate_delivery_date, validate_phone};

fn validate_total_amount_range(value: &Decimal) -> Result<(), ValidationError> {
    let min = Decimal::new(1, 2); // 0.01
    let max = Decimal::new(999_999_999_999, 2); // 对应 DECIMAL(12,2) 的最大值 9_999_999_999.99
    if value < &min || value > &max {
        return Err(ValidationError::new("range"));
    }
    Ok(())
}

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
    pub(crate) created_at: Option<DateTime<Utc>>,

    #[serde(rename = "afterSaleAt")]
    pub(crate) after_sale_at: Option<DateTime<Utc>>,
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
}

/// Order details - comprehensive order information
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct OrderItem {
    pub(crate) id: i32,

    #[serde(rename = "orderCode")]
    pub(crate) order_code: String,

    #[serde(rename = "customerName")]
    pub(crate) customer_name: String,

    #[serde(rename = "orderStatus")]
    pub(crate) order_status: String,

    #[serde(rename = "totalAmount")]
    pub(crate) total_amount: Decimal,

    #[serde(rename = "discountAmount")]
    pub(crate) discount_amount: Decimal,

    #[serde(rename = "actualAmount")]
    pub(crate) actual_amount: Decimal,

    #[serde(rename = "deliveryDate")]
    pub(crate) delivery_date: NaiveDate,

    #[serde(rename = "deliveryAddress")]
    pub(crate) delivery_address: String,

    #[serde(rename = "contactName")]
    pub(crate) contact_name: String,

    #[serde(rename = "contactPhone")]
    pub(crate) contact_phone: String,

    pub(crate) remark: Option<String>,

    #[serde(rename = "createdBy")]
    pub(crate) created_by: String,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,

    #[serde(rename = "confirmBy")]
    pub(crate) confirmed_by: Option<String>,

    #[serde(rename = "confirmedAt")]
    pub(crate) confirmed_at: Option<DateTime<Utc>>,

    #[serde(rename = "processedBy")]
    pub(crate) processed_by: Option<String>,

    #[serde(rename = "processedAt")]
    pub(crate) processed_at: Option<DateTime<Utc>>,

    #[serde(rename = "stockedBy")]
    pub(crate) stocked_by: Option<String>,

    #[serde(rename = "stockedAt")]
    pub(crate) stocked_at: Option<DateTime<Utc>>,

    #[serde(rename = "afterSaleAt")]
    pub(crate) after_sale_at: Option<DateTime<Utc>>,

    #[serde(rename = "rejectBy")]
    pub(crate) rejected_by: Option<String>,

    #[serde(rename = "rejectedAt")]
    pub(crate) rejected_at: Option<DateTime<Utc>>,

    #[serde(rename = "rejectReason")]
    pub(crate) reject_reason: Option<String>,

    #[serde(rename = "completedBy")]
    pub(crate) completed_by: Option<String>,

    #[serde(rename = "deliveryStaffName")]
    pub(crate) delivery_staff_name: Option<String>,

    #[serde(rename = "deliveryStaffPhone")]
    pub(crate) delivery_staff_phone: Option<String>,

    #[serde(rename = "providerName")]
    pub(crate) provider_name: Option<String>,

    #[serde(rename = "completedAt")]
    pub(crate) completed_at: Option<DateTime<Utc>>,
}

/// 订单验收项信息
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderInspectionItem {
    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i32,

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

/// 发货明细项
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderDeliveryItem {
    #[serde(rename = "id")]
    pub(crate) id: i64,

    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i64,

    #[serde(rename = "productCode")]
    pub(crate) product_code: String,

    #[serde(rename = "actualQty")]
    pub(crate) actual_qty: Decimal,

    #[serde(rename = "unitPrice")]
    pub(crate) unit_price: Decimal,

    #[serde(rename = "subtotal")]
    pub(crate) subtotal: Decimal,

    #[serde(rename = "weightUnit")]
    pub(crate) weight_unit: Option<String>,

    #[serde(rename = "remark", skip_serializing_if = "Option::is_none")]
    pub(crate) remark: Option<String>,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub(crate) updated_at: DateTime<Utc>,
}

/// 发货记录
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderDelivery {
    #[serde(rename = "id")]
    pub(crate) id: i64,

    #[serde(rename = "assignmentId")]
    pub(crate) assignment_id: u32,

    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
    pub(crate) parent_id: Option<i64>,

    #[serde(rename = "deliveryRound")]
    pub(crate) delivery_round: i32,

    #[serde(rename = "deliveryType")]
    pub(crate) delivery_type: String,

    #[serde(rename = "deliveredAt", skip_serializing_if = "Option::is_none")]
    pub(crate) delivered_at: Option<DateTime<Utc>>,

    #[serde(rename = "deliveredBy")]
    pub(crate) delivered_by: String,

    #[serde(rename = "deliveryStatus")]
    pub(crate) delivery_status: String,

    #[serde(rename = "remark", skip_serializing_if = "Option::is_none")]
    pub(crate) remark: Option<String>,

    #[serde(rename = "deliveryContactNumber")]
    pub(crate) delivery_contact_number: String,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub(crate) updated_at: DateTime<Utc>,

    #[serde(rename = "items")]
    pub(crate) items: Vec<OrderDeliveryItem>,
}

/// 订单状态变更历史
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderStatusHistory {
    #[serde(rename = "toStatus")]
    pub(crate) to_status: String,

    #[serde(rename = "changeReason", skip_serializing_if = "Option::is_none")]
    pub(crate) change_reason: Option<String>,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,
}


/// 收据响应结构体（按 receipt 分组）
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ReceiptResponse {
    #[serde(rename = "id")]
    pub(crate) id: i32,

    #[serde(rename = "inspectionId")]
    pub(crate) inspection_id: u32,

    #[serde(rename = "operationType")]
    pub(crate) operation_type: ReceiptOperationType,

    #[serde(rename = "status")]
    pub(crate) status: String,

    #[serde(rename = "items")]
    pub(crate) items: Vec<ReceiptItem>,

    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,
}

/// 收据项结构体
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ReceiptItem {
    #[serde(rename = "inspectionId")]
    pub(crate) inspection_id: u32,

    #[serde(rename = "productId")]
    pub(crate) product_id: String,

    #[serde(rename = "productName")]
    pub(crate) product_name: String,

    #[serde(rename = "quantity")]
    pub(crate) quantity: Decimal,

    #[serde(rename = "unit")]
    pub(crate) unit: String,

    #[serde(rename = "reason")]
    pub(crate) reason: String,
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

    pub(crate) unit: String,
    #[serde(rename = "orderedQty")]
    pub(crate) ordered_qty: Decimal,

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
    pub(crate) accepted_at: DateTime<Utc>,
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderReceipt {
    #[serde(rename = "id")]
    pub(crate) order_detail_id: i32,

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

    pub(crate) unit: String,

    #[serde(rename = "processingRequirements")]
    pub(crate) processing_requirements: Option<String>,

   // pub(crate) remark: Option<ProviderOrderRemark>,
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
    pub(crate) delivery_date: Option<DateTime<Utc>>,

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
    pub(crate) delivery_date: Option<DateTime<Utc>>,

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
