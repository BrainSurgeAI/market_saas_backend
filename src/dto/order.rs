use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use sqlx::{prelude::FromRow, types::Decimal};

/// Data transfer object for creating a new order
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CreateOrderDTO {
    #[serde(rename = "totalAmount")]
    pub total_amount: Decimal,

    #[serde(rename = "deliveryInfo")]
    pub delivery_info: DeliveryInfo,

    #[serde(rename = "items")]
    pub items: Vec<CreateOrderItem>,
}

// impl CreateOrderDTO {
//     /// Create a new CreateOrderDTO
//     pub fn new(total_amount: Decimal, delivery_info: DeliveryInfo, items: Vec<CreateOrderItem>) -> Self {
//         Self {
//             total_amount,
//             delivery_info,
//             items,
//         }
//     }

//     /// Calculate total items count
//     pub fn items_count(&self) -> usize {
//         self.items.len()
//     }

//     /// Calculate total quantity across all items
//     pub fn total_quantity(&self) -> Decimal {
//         self.items.iter().map(|item| item.quantity).sum()
//     }

//     /// Validate order has items
//     pub fn is_empty(&self) -> bool {
//         self.items.is_empty()
//     }
// }

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

// impl DeliveryInfo {
//     /// Create a new DeliveryInfo
//     pub fn new(delivery_date: String, delivery_address: String, contact_name: String, contact_phone: String) -> Self {
//         Self {
//             delivery_date,
//             delivery_address,
//             contact_name,
//             contact_phone,
//         }
//     }

//     /// Check if all required fields are non-empty
//     pub fn is_complete(&self) -> bool {
//         !self.delivery_address.is_empty() 
//             && !self.contact_name.is_empty() 
//             && !self.contact_phone.is_empty()
//             && !self.delivery_date.is_empty()
//     }
// }

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

// impl CreateOrderItem {
//     /// Calculate actual discount amount
//     pub fn discount_amount(&self) -> Decimal {
//         self.original_amount - self.total
//     }

//     /// Check if item has discount
//     pub fn has_discount(&self) -> bool {
//         self.discount_rate > Decimal::ZERO
//     }

//     /// Get processing services count
//     pub fn processing_services_count(&self) -> usize {
//         self.processing_services.len()
//     }

//     /// Check if item has processing services
//     pub fn has_processing_services(&self) -> bool {
//         !self.processing_services.is_empty()
//     }
// }

/// Processing service for an order item
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProcessingService {
    #[serde(rename = "type")]
    pub name: String,

    pub description: Option<String>,
}

// impl ProcessingService {
//     /// Create a new ProcessingService
//     pub fn new(name: String, description: Option<String>) -> Self {
//         Self { name, description }
//     }

//     /// Create a simple processing service without description
//     pub fn simple(name: String) -> Self {
//         Self::new(name, None)
//     }
// }

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

// impl OrderResponse {
//     /// Calculate discount amount
//     pub fn discount_amount(&self) -> Decimal {
//         self.total_amount - self.actual_amount
//     }

//     /// Check if order has discount
//     pub fn has_discount(&self) -> bool {
//         self.actual_amount < self.total_amount
//     }

//     /// Get order status as enum
//     pub fn status_enum(&self) -> Result<OrderStatus, crate::common::AppError> {
//         OrderStatus::from_str(&self.order_status)
//     }

//     /// Check if order is in after-sale status
//     pub fn is_after_sale(&self) -> bool {
//         self.after_sale_at.is_some()
//     }
// }

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

// impl OrderQueryParams {
//     /// Create new query params with defaults
//     pub fn new() -> Self {
//         Self::default()
//     }

//     /// Set page number
//     pub fn with_page(mut self, page: i32) -> Self {
//         self.page = Some(page);
//         self
//     }

//     /// Set page size
//     pub fn with_page_size(mut self, page_size: i32) -> Self {
//         self.page_size = Some(page_size);
//         self
//     }

//     /// Set order status filter
//     pub fn with_status(mut self, status: String) -> Self {
//         self.order_status = Some(status);
//         self
//     }

//     /// Get effective page number (default to 1)
//     pub fn effective_page(&self) -> i32 {
//         self.page.unwrap_or(1)
//     }

//     /// Get effective page size (default to 20)
//     pub fn effective_page_size(&self) -> i32 {
//         self.page_size.unwrap_or(20)
//     }
// }

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

// impl OrderItem {
//     /// Get order status as enum
//     pub fn status_enum(&self) -> Result<OrderStatus, crate::common::AppError> {
//         OrderStatus::from_str(&self.order_status)
//     }

//     /// Check if order has discount
//     pub fn has_discount(&self) -> bool {
//         self.discount_amount > Decimal::ZERO
//     }

//     /// Calculate discount rate as percentage
//     pub fn discount_rate(&self) -> Decimal {
//         if self.total_amount > Decimal::ZERO {
//             (self.discount_amount / self.total_amount) * Decimal::from(100)
//         } else {
//             Decimal::ZERO
//         }
//     }

//     /// Check if order is overdue (delivery date has passed)
//     pub fn is_overdue(&self) -> bool {
//         let today = Utc::now().date_naive();
//         self.delivery_date < today && !self.is_completed()
//     }

//     /// Check if order is completed
//     pub fn is_completed(&self) -> bool {
//         matches!(self.status_enum().ok(), Some(OrderStatus::Completed))
//     }

//     /// Check if order is in processing
//     pub fn is_processing(&self) -> bool {
//         matches!(self.status_enum().ok(), Some(OrderStatus::Processing))
//     }

//     /// Check if order is pending
//     pub fn is_pending(&self) -> bool {
//         matches!(self.status_enum().ok(), Some(OrderStatus::Pending))
//     }

//     /// Check if order is confirmed
//     pub fn is_confirmed(&self) -> bool {
//         matches!(self.status_enum().ok(), Some(OrderStatus::Confirmed))
//     }

//     /// Check if order is stocked
//     pub fn is_stocked(&self) -> bool {
//         matches!(self.status_enum().ok(), Some(OrderStatus::Stocked))
//     }

//     /// Check if order is in after sale
//     pub fn is_after_sale(&self) -> bool {
//         matches!(self.status_enum().ok(), Some(OrderStatus::AfterSale))
//     }

//     /// Check if order is rejected
//     pub fn is_rejected(&self) -> bool {
//         matches!(self.status_enum().ok(), Some(OrderStatus::Rejected))
//     }

//     /// Get days since creation
//     pub fn days_since_creation(&self) -> i64 {
//         let now = Utc::now();
//         (now - self.created_at).num_days()
//     }

//     /// Get days until delivery
//     pub fn days_until_delivery(&self) -> i64 {
//         let today = Utc::now().date_naive();
//         (self.delivery_date - today).num_days()
//     }

//     /// Check if order has remark
//     pub fn has_remark(&self) -> bool {
//         self.remark.as_ref().map_or(false, |r| !r.is_empty())
//     }

//     /// Check if order was rejected with reason
//     pub fn has_reject_reason(&self) -> bool {
//         self.reject_reason.as_ref().map_or(false, |r| !r.is_empty())
//     }

//     /// Get processing duration if available
//     pub fn processing_duration(&self) -> Option<chrono::Duration> {
//         match (self.confirmed_at, self.processed_at) {
//             (Some(confirmed), Some(processed)) => Some(processed - confirmed),
//             _ => None,
//         }
//     }

//     /// Get completion duration if available
//     pub fn completion_duration(&self) -> Option<chrono::Duration> {
//         match (self.created_at, self.completed_at) {
//             (created, Some(completed)) => Some(completed - created),
//             _ => None,
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderDetailResponse {
    pub order: OrderItem,

    pub items: Vec<OrderDetail>,

    pub receipts: Vec<OrderReceipt>,
}

// impl OrderDetailResponse {
//     /// Create new order detail response
//     pub fn new(order: OrderItem, items: Vec<OrderDetail>, receipts: Vec<OrderReceipt>) -> Self {
//         Self { order, items, receipts }
//     }

//     /// Get total items count
//     pub fn items_count(&self) -> usize {
//         self.items.len()
//     }

//     /// Get receipts count
//     pub fn receipts_count(&self) -> usize {
//         self.receipts.len()
//     }

//     /// Check if order has receipts
//     pub fn has_receipts(&self) -> bool {
//         !self.receipts.is_empty()
//     }

//     /// Get total quantity across all items
//     pub fn total_quantity(&self) -> Decimal {
//         self.items.iter().map(|item| item.quantity).sum()
//     }

//     /// Get total actual quantity across all items (if available)
//     pub fn total_actual_quantity(&self) -> Decimal {
//         self.items.iter()
//             .filter_map(|item| item.actual_quantity)
//             .sum()
//     }
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub struct OrderDetail {
    pub id: i32,
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

// impl OrderDetail {
//     /// Calculate discount amount
//     pub fn discount_amount(&self) -> Decimal {
//         self.original_price - self.actual_price
//     }

//     /// Check if item has discount
//     pub fn has_discount(&self) -> bool {
//         self.discount_rate > Decimal::ZERO
//     }

//     /// Check if actual quantity differs from ordered quantity
//     pub fn has_quantity_difference(&self) -> bool {
//         self.actual_quantity.map_or(false, |actual| actual != self.quantity)
//     }

//     /// Calculate quantity difference (actual - ordered)
//     pub fn quantity_difference(&self) -> Option<Decimal> {
//         self.actual_quantity.map(|actual| actual - self.quantity)
//     }

//     /// Check if item has processing requirements
//     pub fn has_processing_requirements(&self) -> bool {
//         self.processing_requirements.as_ref().map_or(false, |req| !req.is_empty())
//     }

//     /// Check if item has remark
//     pub fn has_remark(&self) -> bool {
//         self.remark.as_ref().map_or(false, |r| !r.is_empty())
//     }

//     /// Get effective quantity (actual if available, otherwise ordered)
//     pub fn effective_quantity(&self) -> Decimal {
//         self.actual_quantity.unwrap_or(self.quantity)
//     }

//     /// Get effective amount (actual if available, otherwise calculated)
//     pub fn effective_amount(&self) -> Decimal {
//         self.actual_amount.unwrap_or(self.total_amount)
//     }
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct ActualQuantity {
    pub(crate) id: i32,

    #[serde(rename = "actualQuantity")]
    pub(crate) actual_quantity: Decimal,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct DeliverToMarketDTO {
    #[serde(rename = "stockedBy")]
    pub(crate) stocked_by: String,

    pub(crate) items: Vec<ActualQuantity>,
}

// impl ActualQuantityDTO {
//     /// Create new actual quantity DTO
//     pub fn new(stocked_by: String, items: Vec<ActualQuantity>) -> Self {
//         Self { stocked_by, items }
//     }

//     /// Get total items count
//     pub fn items_count(&self) -> usize {
//         self.items.len()
//     }

//     /// Get total actual quantity
//     pub fn total_actual_quantity(&self) -> Decimal {
//         self.items.iter().map(|item| item.actual_quantity).sum()
//     }

//     /// Check if DTO has items
//     pub fn has_items(&self) -> bool {
//         !self.items.is_empty()
//     }
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub struct AcceptedOrderResponseDTO {
    #[serde(rename = "orderCode")]
    pub order_code: String,

    #[serde(rename = "orderStatus")]
    pub order_status: String,

    #[serde(rename = "acceptedAt")]
    pub accepted_at: DateTime<Utc>,
}

// impl AcceptedOrderResponseDTO {
//     /// Create new accepted order response
//     pub fn new(order_code: String, order_status: String, accepted_at: DateTime<Utc>) -> Self {
//         Self {
//             order_code,
//             order_status,
//             accepted_at,
//         }
//     }

//     /// Get order status as enum
//     pub fn status_enum(&self) -> Result<OrderStatus, crate::common::AppError> {
//         OrderStatus::from_str(&self.order_status)
//     }
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderReceiptDTO {
    #[serde(rename = "operateBy")]
    pub operate_by: String,
    pub receipt: OrderReceipt,
}

// impl OrderReceiptDTO {
//     /// Create new order receipt DTO
//     pub fn new(operate_by: String, receipt: OrderReceipt) -> Self {
//         Self { operate_by, receipt }
//     }
// }

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

// impl ProductsSummaryWithOrdersDTO {
//     /// Check if product has quantity
//     pub fn has_quantity(&self) -> bool {
//         self.total_quantity.map_or(false, |q| q > Decimal::ZERO)
//     }

//     /// Get effective quantity (zero if None)
//     pub fn effective_quantity(&self) -> Decimal {
//         self.total_quantity.unwrap_or(Decimal::ZERO)
//     }

//     /// Check if product has processing requirements
//     pub fn has_processing_requirements(&self) -> bool {
//         self.processing_requirements.as_ref().map_or(false, |req| !req.is_empty())
//     }

//     /// Check if product has customer name
//     pub fn has_customer(&self) -> bool {
//         self.customer_name.as_ref().map_or(false, |name| !name.is_empty())
//     }

//     /// Check if product has remark
//     pub fn has_remark(&self) -> bool {
//         self.remark.as_ref().map_or(false, |r| !r.is_empty())
//     }
// }

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

// impl OrderReceipt {
//     /// Create new order receipt
//     pub fn new(
//         order_detail_id: i32,
//         order_code: String,
//         product_code: String,
//         product_name: String,
//         operation_type: ReceiptOperationType,
//         quantity: Decimal,
//         reason: String,
//         unit: String,
//         evidence_images: Option<String>,
//     ) -> Self {
//         Self {
//             order_detail_id,
//             order_code,
//             product_code,
//             product_name,
//             operation_type,
//             quantity,
//             reason,
//             unit,
//             evidence_images,
//         }
//     }

//     /// Check if receipt has evidence images
//     pub fn has_evidence_images(&self) -> bool {
//         self.evidence_images.as_ref().map_or(false, |images| !images.is_empty())
//     }

//     /// Get evidence images as vector (split by comma)
//     pub fn evidence_images_list(&self) -> Vec<String> {
//         self.evidence_images
//             .as_ref()
//             .map(|images| images.split(',').map(|s| s.trim().to_string()).collect())
//             .unwrap_or_default()
//     }

//     /// Check if receipt is for signing
//     pub fn is_sign_receipt(&self) -> bool {
//         matches!(self.operation_type, ReceiptOperationType::Sign)
//     }

//     /// Check if receipt is for return
//     pub fn is_return_receipt(&self) -> bool {
//         matches!(self.operation_type, ReceiptOperationType::Return)
//     }

//     /// Check if receipt is for exchange
//     pub fn is_exchange_receipt(&self) -> bool {
//         matches!(self.operation_type, ReceiptOperationType::Exchange)
//     }
// }

// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// pub(crate) struct UpdateOrderStatusDTO {
//     #[serde(rename = "operateBy")]
//     pub(crate) operate_by: String,

//     pub(crate) status: String,
// }

// impl UpdateOrderStatusDTO {
//     /// Create new update order status DTO
//     pub fn new(operate_by: String, status: String) -> Self {
//         Self { operate_by, status }
//     }

//     /// Get status as enum
//     pub fn status_enum(&self) -> Result<OrderStatus, crate::common::AppError> {
//         OrderStatus::from_str(&self.status)
//     }
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub(crate) struct DispatchOrderDTO {
    #[serde(rename = "providerId")]
    pub(crate) provider_id: i32,

    #[serde(rename = "confirmedBy")]
    pub(crate) confirmed_by: String,
}

// impl DispatchOrderDTO {
//     /// Create new dispatch order DTO
//     pub fn new(provider_id: i32, confirmed_by: String) -> Self {
//         Self {
//             provider_id,
//             confirmed_by,
//         }
//     }
// }

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

// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// pub enum OrderStatus {
//     // default status, when order is created
//     #[serde(rename = "PENDING")]
//     Pending,

//     // when order is confirmed by warehouse, provider is ready to dispatch
//     #[serde(rename = "CONFIRMED")]
//     Confirmed,

//     // provider is processing the order
//     #[serde(rename = "PROCESSING")]
//     Processing,

//     // when order is stocked by provider
//     #[serde(rename = "STOCKED")]
//     Stocked,

//     // when order is after sale
//     #[serde(rename = "AFTER_SALE")]
//     AfterSale,

//     // when order is rejected by customer
//     #[serde(rename = "REJECTED")]
//     Rejected,

//     // when order is completed by customer
//     #[serde(rename = "COMPLETED")]
//     Completed,
// }

// impl OrderStatus {
//     /// Check if status transition is valid
//     ///
//     /// # Arguments
//     ///
//     /// * `from` - Current status string representation
//     /// * `to` - Target status string representation
//     ///
//     /// # Returns
//     ///
//     /// Returns `Ok(())` if transition is valid, otherwise returns `Err(AppError)`
//     pub fn validate_transition(from: &str, to: &str) -> Result<(), crate::common::AppError> {
//         use crate::common::AppError;

//         // Convert current status and target status to enums
//         let from_status = Self::from_str(from)?;
//         let to_status = Self::from_str(to)?;

//         // Define valid status transitions
//         let valid = match from_status {
//             Self::Pending => matches!(to_status, Self::Confirmed),

//             Self::Confirmed => matches!(to_status, Self::Processing),

//             Self::Processing => matches!(to_status, Self::Stocked),

//             Self::Stocked => matches!(to_status, Self::Completed | Self::AfterSale),

//             Self::AfterSale => matches!(to_status, Self::Completed | Self::Rejected),

//             Self::Rejected => matches!(to_status, Self::Completed),

//             Self::Completed => false, // Completed status is terminal state
//         };

//         if valid {
//             Ok(())
//         } else {
//             Err(AppError::Validation(format!(
//                 "无效的状态: 从 {} 到 {}",
//                 from, to
//             )))
//         }
//     }

//     /// Check if status transition is valid and validate tenant type permissions
//     ///
//     /// # Arguments
//     ///
//     /// * `from` - Current status string representation
//     /// * `to` - Target status string representation
//     /// * `tenant_type` - Tenant type attempting to execute the status transition
//     ///
//     /// # Returns
//     ///
//     /// Returns `Ok(())` if transition is valid and tenant has permission, otherwise returns `Err(AppError)`
//     pub fn validate_transition_with_tenant(
//         from: &str,
//         to: &str,
//         tenant_type: &str,
//     ) -> Result<(), crate::common::AppError> {
//         use crate::common::AppError;
//         use crate::repositories::TenantType;

//         // First validate if status transition follows business rules
//         Self::validate_transition(from, to)?;

//         // Then validate if tenant has permission to execute this status transition
//         let to_status = Self::from_str(to)?;
//         let tenant = match tenant_type {
//             t if t == TenantType::Customer.to_string() => TenantType::Customer,
//             t if t == TenantType::Provider.to_string() => TenantType::Provider,
//             t if t == TenantType::Market.to_string() => TenantType::Market,
//             _ => {
//                 return Err(AppError::Validation(format!(
//                     "无效的租户类型: {}",
//                     tenant_type
//                 )))
//             }
//         };

//         // Check tenant permissions
//         match to_status {
//             Self::Confirmed => {
//                 // Only market administrators can confirm orders
//                 if !matches!(tenant, TenantType::Market) {
//                     return Err(AppError::Forbidden(
//                         "只有市场管理员才能确认订单".to_string(),
//                     ));
//                 }
//             }
//             Self::Processing => {
//                 // Only providers can start processing orders
//                 if !matches!(tenant, TenantType::Provider) {
//                     return Err(AppError::Forbidden(
//                         "只有供应商才能开始处理订单".to_string(),
//                     ));
//                 }
//             }
//             Self::Stocked => {
//                 // Only providers can confirm order stocking
//                 if !matches!(tenant, TenantType::Provider) {
//                     return Err(AppError::Forbidden(
//                         "只有供应商才能确认订单入库".to_string(),
//                     ));
//                 }
//             }
//             Self::AfterSale => {
//                 // Only customers can mark orders as after-sale
//                 if !matches!(tenant, TenantType::Customer) {
//                     return Err(AppError::Forbidden(
//                         "只有客户才能将订单标记为售后".to_string(),
//                     ));
//                 }
//             }
//             Self::Rejected => {
//                 // Only customers can reject after-sale requests
//                 if !matches!(tenant, TenantType::Customer) {
//                     return Err(AppError::Forbidden("只有客户才能拒绝售后请求".to_string()));
//                 }
//             }
//             Self::Completed => {
//                 // Only customers can confirm order completion
//                 if !matches!(tenant, TenantType::Customer) {
//                     return Err(AppError::Forbidden("只有客户才能确认订单完成".to_string()));
//                 }
//             }
//             _ => {}
//         }

//         Ok(())
//     }
// }

// impl FromStr for OrderStatus {
//     type Err = crate::common::AppError;

//     fn from_str(status: &str) -> Result<Self, Self::Err> {
//         use crate::common::AppError;

//         match status {
//             "PENDING" => Ok(Self::Pending),
//             "CONFIRMED" => Ok(Self::Confirmed),
//             "PROCESSING" => Ok(Self::Processing),
//             "STOCKED" => Ok(Self::Stocked),
//             "AFTER_SALE" => Ok(Self::AfterSale),
//             "REJECTED" => Ok(Self::Rejected),
//             "COMPLETED" => Ok(Self::Completed),
//             _ => Err(AppError::Validation(format!("无效的订单状态: {}", status))),
//         }
//     }
// }