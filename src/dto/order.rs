use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Decimal};
use validator::{Validate, ValidationError};

use crate::{dto::{validate_delivery_date, validate_phone}, repositories::orders::common::OrderBaseInfoResponse};

fn validate_total_amount_range(value: &Decimal) -> Result<(), ValidationError> {
    let min = Decimal::new(1, 2); // 0.01
    let max = Decimal::new(999_999_999_999, 2); // 对应 DECIMAL(12,2) 的最大值 9_999_999_999.99
    if value < &min || value > &max {
        return Err(ValidationError::new("range"));
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OrderDetailResponse {
    pub(crate) order: OrderBaseInfoResponse,

    pub(crate) items: Vec<OrderDetail>,

    pub(crate) receipts: Vec<ReceiptResponse>,
}

/// 收据响应结构体（按 receipt 分组）
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ReceiptResponse {
    #[serde(rename = "id")]
    pub(crate) id: i32,

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

    #[serde(rename = "deliveredQuantity")]
    pub(crate) delivered_quantity: Option<Decimal>,

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

    #[serde(rename = "marketInspectedQuantity")]
    pub(crate) market_inspected_quantity: Option<Decimal>,

    #[serde(rename = "customerInspectedQuantity")]
    pub(crate) customer_inspected_quantity: Option<Decimal>,
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct DeliverQuantity {
    pub(crate) id: i32,

    #[serde(rename = "deliveredQuantity")]
    pub(crate) delivered_quantity: Decimal,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct DeliverToMarketDTO {
    #[serde(rename = "stockedBy")]
    pub(crate) stocked_by: String,

    pub(crate) items: Vec<DeliverQuantity>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct ExchangeDTO {
    #[serde(rename = "operateBy")]
    pub(crate) operate_by: String,

    pub(crate) items: Vec<ExchangeItem>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ExchangeItem {
    // order detail id
    #[serde(rename = "id")]
    pub(crate) id: i32,

    #[serde(rename = "productId")]
    pub(crate) product_code: String,

    #[serde(rename = "actualQuantity")]
    pub(crate) actual_quantity: Decimal,

    #[serde(rename = "requestedQuantity")]
    pub(crate) requested_quantity: Decimal,

    pub(crate) remark: Option<String>,
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
    #[serde(rename = "operateBy")]
    pub(crate) operate_by: String,
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
pub(crate) struct ExchangeItemUpdateDTO {
    #[serde(rename = "orderDetailId")]
    pub(crate) order_detail_id: i32,

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
