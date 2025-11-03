use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;
use sqlx::{prelude::FromRow, types::Decimal};

/// Data transfer object for creating a new order
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub struct CreateOrderDTO {
    #[serde(rename = "totalAmount")]
    pub total_amount: Decimal,

    #[serde(rename = "deliveryInfo")]
    pub delivery_info: DeliveryInfo,

    #[serde(rename = "items")]
    pub items: Vec<CreateOrderItem>,
}

/// Delivery information for an order
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeliveryInfo {
    #[serde(rename = "deliveryDate")]
    pub delivery_date: String,

    #[serde(rename = "address")]
    pub delivery_address: String,

    #[serde(rename = "contactName")]
    pub contact_name: String,

    #[serde(rename = "contactPhone")]
    pub contact_phone: String,
}


/// Individual item in an order creation request
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub struct CreateOrderItem {
    #[serde(rename = "productId")]
    pub product_code: String,

    #[serde(rename = "name")]
    pub product_name: String,

    #[serde(rename = "categoryId")]
    pub category_id: i32,

    #[serde(rename = "category")]
    pub category_name: String,

    #[serde(rename = "unit")]
    pub unit: String,

    #[serde(rename = "quantity")]
    pub quantity: Decimal,

    #[serde(rename = "price")]
    pub price: Decimal,

    #[serde(rename = "originalPrice")]
    pub original_price: Decimal,

    #[serde(rename = "originalTotal")]
    pub original_amount: Decimal,

    #[serde(rename = "discountRate")]
    pub discount_rate: Decimal,

    #[serde(rename = "total")]
    pub total: Decimal,

    #[serde(rename = "customNote")]
    pub remark: Option<String>,

    #[serde(rename = "processingServices")]
    pub processing_services: Vec<ProcessingService>,
}

/// Processing service for an order item
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProcessingService {
    #[serde(rename = "type")]
    pub name: String,

    pub description: Option<String>,
}

/// Response DTO for order operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct OrderResponse {
    #[serde(rename = "orderCode")]
    pub order_code: String,

    #[serde(rename = "totalAmount")]
    pub total_amount: Decimal,

    #[serde(rename = "actualAmount")]
    pub actual_amount: Decimal,

    #[serde(rename = "deliveryDate")]
    pub delivery_date: NaiveDate,

    #[serde(rename = "deliveryAddress")]
    pub delivery_address: String,

    #[serde(rename = "orderStatus")]
    pub order_status: String,

    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(rename = "afterSaleAt")]
    pub after_sale_at: Option<DateTime<Utc>>,
}

/// Query parameters for order listing
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct OrderQueryParams {
    #[serde(rename = "page")]
    pub page: Option<i32>,

    #[serde(rename = "pageSize")]
    pub page_size: Option<i32>,

    #[serde(rename = "orderStatus")]
    pub order_status: Option<String>,
}


/// Order details - comprehensive order information
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub struct OrderItem {
    pub id: i32,

    #[serde(rename = "orderCode")]
    pub order_code: String,

    #[serde(rename = "customerName")]
    pub customer_name: String,

    #[serde(rename = "orderStatus")]
    pub order_status: String,

    #[serde(rename = "totalAmount")]
    pub total_amount: Decimal,

    #[serde(rename = "discountAmount")]
    pub discount_amount: Decimal,

    #[serde(rename = "actualAmount")]
    pub actual_amount: Decimal,

    #[serde(rename = "deliveryDate")]
    pub delivery_date: NaiveDate,

    #[serde(rename = "deliveryAddress")]
    pub delivery_address: String,

    #[serde(rename = "contactName")]
    pub contact_name: String,

    #[serde(rename = "contactPhone")]
    pub contact_phone: String,

    pub remark: Option<String>,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "confirmBy")]
    pub confirmed_by: Option<String>,

    #[serde(rename = "confirmedAt")]
    pub confirmed_at: Option<DateTime<Utc>>,

    #[serde(rename = "processedBy")]
    pub processed_by: Option<String>,

    #[serde(rename = "processedAt")]
    pub processed_at: Option<DateTime<Utc>>,

    #[serde(rename = "stockedBy")]
    pub stocked_by: Option<String>,

    #[serde(rename = "stockedAt")]
    pub stocked_at: Option<DateTime<Utc>>,

    #[serde(rename = "afterSaleAt")]
    pub after_sale_at: Option<DateTime<Utc>>,

    #[serde(rename = "rejectBy")]
    pub rejected_by: Option<String>,

    #[serde(rename = "rejectedAt")]
    pub rejected_at: Option<DateTime<Utc>>,

    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<String>,

    #[serde(rename = "completedBy")]
    pub completed_by: Option<String>,

    #[serde(rename = "deliveryStaffName")]
    pub delivery_staff_name: Option<String>,

    #[serde(rename = "deliveryStaffPhone")]
    pub delivery_staff_phone: Option<String>,

    #[serde(rename = "providerName")]
    pub provider_name: Option<String>,

    #[serde(rename = "completedAt")]
    pub completed_at: Option<DateTime<Utc>>,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderDetailResponse {
    pub order: OrderItem,

    pub items: Vec<OrderDetail>,

    pub receipts: Vec<OrderReceipt>,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub struct OrderDetail {
    pub id: i32,

    #[serde(rename = "receiptQuantity")]
    pub receipt_quantity: Option<Decimal>,

    #[serde(rename = "productId")]
    pub product_code: String,

    #[serde(rename = "name")]
    pub product_name: String,

    #[serde(rename = "categoryId")]
    pub category_id: i32,

    #[serde(rename = "category")]
    pub category_name: String,

    pub unit: String,
    pub quantity: Decimal,

    #[serde(rename = "price")]
    pub original_price: Decimal,

    #[serde(rename = "discountRate")]
    pub discount_rate: Decimal,

    #[serde(rename = "actualPrice")]
    pub actual_price: Decimal,

    #[serde(rename = "actualQuantity")]
    pub actual_quantity: Option<Decimal>,

    #[serde(rename = "actualAmount")]
    pub actual_amount: Option<Decimal>,

    #[serde(rename = "total")]
    pub total_amount: Decimal,

    #[serde(rename = "processingRequirements")]
    pub processing_requirements: Option<String>,

    pub remark: Option<String>,

    #[serde(rename = "status")]
    pub status: Option<String>,
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
pub(crate) struct ActualQuantity {
    pub(crate) id: i32,

    #[serde(rename = "actualQuantity")]
    pub(crate) actual_quantity: Decimal,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct DeliverToMarketDTO {
    #[serde(rename = "stockedBy")]
    pub(crate) stocked_by: String,

    pub(crate) items: Vec<ActualQuantity>,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub(crate) struct ExchangeDTO {
    #[serde(rename = "operateBy")]
    pub(crate) operate_by: String,

    pub(crate) items: Vec<ExchangeItem>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ExchangeItem {
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
pub struct AcceptedOrderResponseDTO {
    #[serde(rename = "orderCode")]
    pub order_code: String,

    #[serde(rename = "orderStatus")]
    pub order_status: String,

    #[serde(rename = "acceptedAt")]
    pub accepted_at: DateTime<Utc>,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
pub struct OrderReceiptDTO {
    #[serde(rename = "operateBy")]
    pub operate_by: String,
    pub receipt: OrderReceipt,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub struct ProductsSummaryWithOrdersDTO {
    #[serde(rename = "productId")]
    pub product_code: String,

    #[serde(rename = "productName")]
    pub product_name: String,

    #[serde(rename = "totalQuantity")]
    pub total_quantity: Option<Decimal>,

    #[serde(rename = "unit")]
    pub unit: String,

    #[serde(rename = "processingRequirements")]
    pub processing_requirements: Option<String>,

    #[serde(rename = "customerName")]
    pub customer_name: Option<String>,

    pub remark: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderReceipt {
    #[serde(rename = "id")]
    pub order_detail_id: i32,

    #[serde(rename = "orderId")]
    pub order_code: String,

    #[serde(rename = "productId")]
    pub product_code: String,

    #[serde(rename = "productName")]
    pub product_name: String,

    #[serde(rename = "operationType")]
    pub operation_type: ReceiptOperationType,

    pub quantity: Decimal,

    pub reason: String,

    pub unit: String,

    #[serde(skip_serializing_if = "Option::is_none", rename = "evidenceImages")]
    pub evidence_images: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow, Validate)]
pub(crate) struct DispatchOrderDTO {
    #[serde(rename = "providerId")]
    pub(crate) provider_id: i32,

    #[serde(rename = "confirmedBy")]
    pub(crate) confirmed_by: String,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ReceiptOperationType {
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