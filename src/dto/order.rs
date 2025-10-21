use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use sqlx::{prelude::FromRow, types::Decimal};

use std::str::FromStr;

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

impl CreateOrderDTO {
    /// Create a new CreateOrderDTO
    pub fn new(total_amount: Decimal, delivery_info: DeliveryInfo, items: Vec<CreateOrderItem>) -> Self {
        Self {
            total_amount,
            delivery_info,
            items,
        }
    }

    /// Calculate total items count
    pub fn items_count(&self) -> usize {
        self.items.len()
    }

    /// Calculate total quantity across all items
    pub fn total_quantity(&self) -> Decimal {
        self.items.iter().map(|item| item.quantity).sum()
    }

    /// Validate order has items
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
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

impl DeliveryInfo {
    /// Create a new DeliveryInfo
    pub fn new(delivery_date: String, delivery_address: String, contact_name: String, contact_phone: String) -> Self {
        Self {
            delivery_date,
            delivery_address,
            contact_name,
            contact_phone,
        }
    }

    /// Check if all required fields are non-empty
    pub fn is_complete(&self) -> bool {
        !self.delivery_address.is_empty() 
            && !self.contact_name.is_empty() 
            && !self.contact_phone.is_empty()
            && !self.delivery_date.is_empty()
    }
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

impl CreateOrderItem {
    /// Calculate actual discount amount
    pub fn discount_amount(&self) -> Decimal {
        self.original_amount - self.total
    }

    /// Check if item has discount
    pub fn has_discount(&self) -> bool {
        self.discount_rate > Decimal::ZERO
    }

    /// Get processing services count
    pub fn processing_services_count(&self) -> usize {
        self.processing_services.len()
    }

    /// Check if item has processing services
    pub fn has_processing_services(&self) -> bool {
        !self.processing_services.is_empty()
    }
}

/// Processing service for an order item
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProcessingService {
    #[serde(rename = "type")]
    pub name: String,

    pub description: Option<String>,
}

impl ProcessingService {
    /// Create a new ProcessingService
    pub fn new(name: String, description: Option<String>) -> Self {
        Self { name, description }
    }

    /// Create a simple processing service without description
    pub fn simple(name: String) -> Self {
        Self::new(name, None)
    }
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

impl OrderResponse {
    /// Calculate discount amount
    pub fn discount_amount(&self) -> Decimal {
        self.total_amount - self.actual_amount
    }

    /// Check if order has discount
    pub fn has_discount(&self) -> bool {
        self.actual_amount < self.total_amount
    }

    /// Get order status as enum
    pub fn status_enum(&self) -> Result<OrderStatus, crate::common::AppError> {
        OrderStatus::from_str(&self.order_status)
    }

    /// Check if order is in after-sale status
    pub fn is_after_sale(&self) -> bool {
        self.after_sale_at.is_some()
    }
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

impl OrderQueryParams {
    /// Create new query params with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set page number
    pub fn with_page(mut self, page: i32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set page size
    pub fn with_page_size(mut self, page_size: i32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Set order status filter
    pub fn with_status(mut self, status: String) -> Self {
        self.order_status = Some(status);
        self
    }

    /// Get effective page number (default to 1)
    pub fn effective_page(&self) -> i32 {
        self.page.unwrap_or(1)
    }

    /// Get effective page size (default to 20)
    pub fn effective_page_size(&self) -> i32 {
        self.page_size.unwrap_or(20)
    }
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

impl OrderItem {
    /// Get order status as enum
    pub fn status_enum(&self) -> Result<OrderStatus, crate::common::AppError> {
        OrderStatus::from_str(&self.order_status)
    }

    /// Check if order has discount
    pub fn has_discount(&self) -> bool {
        self.discount_amount > Decimal::ZERO
    }

    /// Calculate discount rate as percentage
    pub fn discount_rate(&self) -> Decimal {
        if self.total_amount > Decimal::ZERO {
            (self.discount_amount / self.total_amount) * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    }

    /// Check if order is overdue (delivery date has passed)
    pub fn is_overdue(&self) -> bool {
        let today = Utc::now().date_naive();
        self.delivery_date < today && !self.is_completed()
    }

    /// Check if order is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status_enum().ok(), Some(OrderStatus::Completed))
    }

    /// Check if order is in processing
    pub fn is_processing(&self) -> bool {
        matches!(self.status_enum().ok(), Some(OrderStatus::Processing))
    }

    /// Check if order is pending
    pub fn is_pending(&self) -> bool {
        matches!(self.status_enum().ok(), Some(OrderStatus::Pending))
    }

    /// Check if order is confirmed
    pub fn is_confirmed(&self) -> bool {
        matches!(self.status_enum().ok(), Some(OrderStatus::Confirmed))
    }

    /// Check if order is stocked
    pub fn is_stocked(&self) -> bool {
        matches!(self.status_enum().ok(), Some(OrderStatus::Stocked))
    }

    /// Check if order is in after sale
    pub fn is_after_sale(&self) -> bool {
        matches!(self.status_enum().ok(), Some(OrderStatus::AfterSale))
    }

    /// Check if order is rejected
    pub fn is_rejected(&self) -> bool {
        matches!(self.status_enum().ok(), Some(OrderStatus::Rejected))
    }

    /// Get days since creation
    pub fn days_since_creation(&self) -> i64 {
        let now = Utc::now();
        (now - self.created_at).num_days()
    }

    /// Get days until delivery
    pub fn days_until_delivery(&self) -> i64 {
        let today = Utc::now().date_naive();
        (self.delivery_date - today).num_days()
    }

    /// Check if order has remark
    pub fn has_remark(&self) -> bool {
        self.remark.as_ref().map_or(false, |r| !r.is_empty())
    }

    /// Check if order was rejected with reason
    pub fn has_reject_reason(&self) -> bool {
        self.reject_reason.as_ref().map_or(false, |r| !r.is_empty())
    }

    /// Get processing duration if available
    pub fn processing_duration(&self) -> Option<chrono::Duration> {
        match (self.confirmed_at, self.processed_at) {
            (Some(confirmed), Some(processed)) => Some(processed - confirmed),
            _ => None,
        }
    }

    /// Get completion duration if available
    pub fn completion_duration(&self) -> Option<chrono::Duration> {
        match (self.created_at, self.completed_at) {
            (created, Some(completed)) => Some(completed - created),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderDetailResponse {
    pub order: OrderItem,

    pub items: Vec<OrderDetail>,

    pub receipts: Vec<OrderReceipt>,
}

impl OrderDetailResponse {
    /// Create new order detail response
    pub fn new(order: OrderItem, items: Vec<OrderDetail>, receipts: Vec<OrderReceipt>) -> Self {
        Self { order, items, receipts }
    }

    /// Get total items count
    pub fn items_count(&self) -> usize {
        self.items.len()
    }

    /// Get receipts count
    pub fn receipts_count(&self) -> usize {
        self.receipts.len()
    }

    /// Check if order has receipts
    pub fn has_receipts(&self) -> bool {
        !self.receipts.is_empty()
    }

    /// Get total quantity across all items
    pub fn total_quantity(&self) -> Decimal {
        self.items.iter().map(|item| item.quantity).sum()
    }

    /// Get total actual quantity across all items (if available)
    pub fn total_actual_quantity(&self) -> Decimal {
        self.items.iter()
            .filter_map(|item| item.actual_quantity)
            .sum()
    }
}

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

impl OrderDetail {
    /// Calculate discount amount
    pub fn discount_amount(&self) -> Decimal {
        self.original_price - self.actual_price
    }

    /// Check if item has discount
    pub fn has_discount(&self) -> bool {
        self.discount_rate > Decimal::ZERO
    }

    /// Check if actual quantity differs from ordered quantity
    pub fn has_quantity_difference(&self) -> bool {
        self.actual_quantity.map_or(false, |actual| actual != self.quantity)
    }

    /// Calculate quantity difference (actual - ordered)
    pub fn quantity_difference(&self) -> Option<Decimal> {
        self.actual_quantity.map(|actual| actual - self.quantity)
    }

    /// Check if item has processing requirements
    pub fn has_processing_requirements(&self) -> bool {
        self.processing_requirements.as_ref().map_or(false, |req| !req.is_empty())
    }

    /// Check if item has remark
    pub fn has_remark(&self) -> bool {
        self.remark.as_ref().map_or(false, |r| !r.is_empty())
    }

    /// Get effective quantity (actual if available, otherwise ordered)
    pub fn effective_quantity(&self) -> Decimal {
        self.actual_quantity.unwrap_or(self.quantity)
    }

    /// Get effective amount (actual if available, otherwise calculated)
    pub fn effective_amount(&self) -> Decimal {
        self.actual_amount.unwrap_or(self.total_amount)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub struct ActualQuantity {
    pub id: i32,

    #[serde(rename = "actualQuantity")]
    pub actual_quantity: Decimal,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ActualQuantityDTO {
    #[serde(rename = "stockedBy")]
    pub stocked_by: String,

    pub items: Vec<ActualQuantity>,
}

impl ActualQuantityDTO {
    /// Create new actual quantity DTO
    pub fn new(stocked_by: String, items: Vec<ActualQuantity>) -> Self {
        Self { stocked_by, items }
    }

    /// Get total items count
    pub fn items_count(&self) -> usize {
        self.items.len()
    }

    /// Get total actual quantity
    pub fn total_actual_quantity(&self) -> Decimal {
        self.items.iter().map(|item| item.actual_quantity).sum()
    }

    /// Check if DTO has items
    pub fn has_items(&self) -> bool {
        !self.items.is_empty()
    }
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

impl AcceptedOrderResponseDTO {
    /// Create new accepted order response
    pub fn new(order_code: String, order_status: String, accepted_at: DateTime<Utc>) -> Self {
        Self {
            order_code,
            order_status,
            accepted_at,
        }
    }

    /// Get order status as enum
    pub fn status_enum(&self) -> Result<OrderStatus, crate::common::AppError> {
        OrderStatus::from_str(&self.order_status)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderReceiptDTO {
    #[serde(rename = "operateBy")]
    pub operate_by: String,
    pub receipt: OrderReceipt,
}

impl OrderReceiptDTO {
    /// Create new order receipt DTO
    pub fn new(operate_by: String, receipt: OrderReceipt) -> Self {
        Self { operate_by, receipt }
    }
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

impl ProductsSummaryWithOrdersDTO {
    /// Check if product has quantity
    pub fn has_quantity(&self) -> bool {
        self.total_quantity.map_or(false, |q| q > Decimal::ZERO)
    }

    /// Get effective quantity (zero if None)
    pub fn effective_quantity(&self) -> Decimal {
        self.total_quantity.unwrap_or(Decimal::ZERO)
    }

    /// Check if product has processing requirements
    pub fn has_processing_requirements(&self) -> bool {
        self.processing_requirements.as_ref().map_or(false, |req| !req.is_empty())
    }

    /// Check if product has customer name
    pub fn has_customer(&self) -> bool {
        self.customer_name.as_ref().map_or(false, |name| !name.is_empty())
    }

    /// Check if product has remark
    pub fn has_remark(&self) -> bool {
        self.remark.as_ref().map_or(false, |r| !r.is_empty())
    }
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

impl OrderReceipt {
    /// Create new order receipt
    pub fn new(
        order_detail_id: i32,
        order_code: String,
        product_code: String,
        product_name: String,
        operation_type: ReceiptOperationType,
        quantity: Decimal,
        reason: String,
        unit: String,
        evidence_images: Option<String>,
    ) -> Self {
        Self {
            order_detail_id,
            order_code,
            product_code,
            product_name,
            operation_type,
            quantity,
            reason,
            unit,
            evidence_images,
        }
    }

    /// Check if receipt has evidence images
    pub fn has_evidence_images(&self) -> bool {
        self.evidence_images.as_ref().map_or(false, |images| !images.is_empty())
    }

    /// Get evidence images as vector (split by comma)
    pub fn evidence_images_list(&self) -> Vec<String> {
        self.evidence_images
            .as_ref()
            .map(|images| images.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }

    /// Check if receipt is for signing
    pub fn is_sign_receipt(&self) -> bool {
        matches!(self.operation_type, ReceiptOperationType::Sign)
    }

    /// Check if receipt is for return
    pub fn is_return_receipt(&self) -> bool {
        matches!(self.operation_type, ReceiptOperationType::Return)
    }

    /// Check if receipt is for exchange
    pub fn is_exchange_receipt(&self) -> bool {
        matches!(self.operation_type, ReceiptOperationType::Exchange)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UpdateOrderStatusDTO {
    #[serde(rename = "operateBy")]
    pub operate_by: String,

    pub status: String,
}

impl UpdateOrderStatusDTO {
    /// Create new update order status DTO
    pub fn new(operate_by: String, status: String) -> Self {
        Self { operate_by, status }
    }

    /// Get status as enum
    pub fn status_enum(&self) -> Result<OrderStatus, crate::common::AppError> {
        OrderStatus::from_str(&self.status)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, FromRow)]
pub struct DispatchOrderDTO {
    #[serde(rename = "providerId")]
    pub provider_id: i32,

    #[serde(rename = "confirmedBy")]
    pub confirmed_by: String,
}

impl DispatchOrderDTO {
    /// Create new dispatch order DTO
    pub fn new(provider_id: i32, confirmed_by: String) -> Self {
        Self {
            provider_id,
            confirmed_by,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OrderStatus {
    // default status, when order is created
    #[serde(rename = "PENDING")]
    Pending,

    // when order is confirmed by warehouse, provider is ready to dispatch
    #[serde(rename = "CONFIRMED")]
    Confirmed,

    // provider is processing the order
    #[serde(rename = "PROCESSING")]
    Processing,

    // when order is stocked by provider
    #[serde(rename = "STOCKED")]
    Stocked,

    // when order is after sale
    #[serde(rename = "AFTER_SALE")]
    AfterSale,

    // when order is rejected by customer
    #[serde(rename = "REJECTED")]
    Rejected,

    // when order is completed by customer
    #[serde(rename = "COMPLETED")]
    Completed,
}

impl OrderStatus {
    /// Check if status transition is valid
    ///
    /// # Arguments
    ///
    /// * `from` - Current status string representation
    /// * `to` - Target status string representation
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if transition is valid, otherwise returns `Err(AppError)`
    pub fn validate_transition(from: &str, to: &str) -> Result<(), crate::common::AppError> {
        use crate::common::AppError;

        // Convert current status and target status to enums
        let from_status = Self::from_str(from)?;
        let to_status = Self::from_str(to)?;

        // Define valid status transitions
        let valid = match from_status {
            Self::Pending => matches!(to_status, Self::Confirmed),

            Self::Confirmed => matches!(to_status, Self::Processing),

            Self::Processing => matches!(to_status, Self::Stocked),

            Self::Stocked => matches!(to_status, Self::Completed | Self::AfterSale),

            Self::AfterSale => matches!(to_status, Self::Completed | Self::Rejected),

            Self::Rejected => matches!(to_status, Self::Completed),

            Self::Completed => false, // Completed status is terminal state
        };

        if valid {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "无效的状态: 从 {} 到 {}",
                from, to
            )))
        }
    }

    /// Check if status transition is valid and validate tenant type permissions
    ///
    /// # Arguments
    ///
    /// * `from` - Current status string representation
    /// * `to` - Target status string representation
    /// * `tenant_type` - Tenant type attempting to execute the status transition
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if transition is valid and tenant has permission, otherwise returns `Err(AppError)`
    pub fn validate_transition_with_tenant(
        from: &str,
        to: &str,
        tenant_type: &str,
    ) -> Result<(), crate::common::AppError> {
        use crate::common::AppError;
        use crate::repositories::TenantType;

        // First validate if status transition follows business rules
        Self::validate_transition(from, to)?;

        // Then validate if tenant has permission to execute this status transition
        let to_status = Self::from_str(to)?;
        let tenant = match tenant_type {
            t if t == TenantType::Customer.to_string() => TenantType::Customer,
            t if t == TenantType::Provider.to_string() => TenantType::Provider,
            t if t == TenantType::Market.to_string() => TenantType::Market,
            _ => {
                return Err(AppError::Validation(format!(
                    "无效的租户类型: {}",
                    tenant_type
                )))
            }
        };

        // Check tenant permissions
        match to_status {
            Self::Confirmed => {
                // Only market administrators can confirm orders
                if !matches!(tenant, TenantType::Market) {
                    return Err(AppError::Forbidden(
                        "只有市场管理员才能确认订单".to_string(),
                    ));
                }
            }
            Self::Processing => {
                // Only providers can start processing orders
                if !matches!(tenant, TenantType::Provider) {
                    return Err(AppError::Forbidden(
                        "只有供应商才能开始处理订单".to_string(),
                    ));
                }
            }
            Self::Stocked => {
                // Only providers can confirm order stocking
                if !matches!(tenant, TenantType::Provider) {
                    return Err(AppError::Forbidden(
                        "只有供应商才能确认订单入库".to_string(),
                    ));
                }
            }
            Self::AfterSale => {
                // Only customers can mark orders as after-sale
                if !matches!(tenant, TenantType::Customer) {
                    return Err(AppError::Forbidden(
                        "只有客户才能将订单标记为售后".to_string(),
                    ));
                }
            }
            Self::Rejected => {
                // Only customers can reject after-sale requests
                if !matches!(tenant, TenantType::Customer) {
                    return Err(AppError::Forbidden("只有客户才能拒绝售后请求".to_string()));
                }
            }
            Self::Completed => {
                // Only customers can confirm order completion
                if !matches!(tenant, TenantType::Customer) {
                    return Err(AppError::Forbidden("只有客户才能确认订单完成".to_string()));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl FromStr for OrderStatus {
    type Err = crate::common::AppError;

    fn from_str(status: &str) -> Result<Self, Self::Err> {
        use crate::common::AppError;

        match status {
            "PENDING" => Ok(Self::Pending),
            "CONFIRMED" => Ok(Self::Confirmed),
            "PROCESSING" => Ok(Self::Processing),
            "STOCKED" => Ok(Self::Stocked),
            "AFTER_SALE" => Ok(Self::AfterSale),
            "REJECTED" => Ok(Self::Rejected),
            "COMPLETED" => Ok(Self::Completed),
            _ => Err(AppError::Validation(format!("无效的订单状态: {}", status))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;
    use serde_json::json;
    use std::str::FromStr;

    // Test constants for better maintainability
    const VALID_PRODUCT_CODE: &str = "P001";
    const VALID_ORDER_CODE: &str = "ORD001";
    const VALID_CATEGORY_ID: i32 = 1;
    const VALID_TOTAL_AMOUNT: &str = "100.50";
    const VALID_DELIVERY_DATE: &str = "2024-12-31";
    const VALID_PHONE: &str = "13800138000";

    // Helper functions to create test data
    fn create_decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    fn create_test_delivery_info() -> DeliveryInfo {
        DeliveryInfo {
            delivery_date: VALID_DELIVERY_DATE.to_string(),
            delivery_address: "Test Address".to_string(),
            contact_name: "John Doe".to_string(),
            contact_phone: VALID_PHONE.to_string(),
        }
    }

    fn create_test_processing_service() -> ProcessingService {
        ProcessingService {
            name: "切片".to_string(),
            description: Some("薄切片".to_string()),
        }
    }

    fn create_test_order_item() -> CreateOrderItem {
        CreateOrderItem {
            product_code: VALID_PRODUCT_CODE.to_string(),
            product_name: "Apple".to_string(),
            category_id: VALID_CATEGORY_ID,
            category_name: "Fruits".to_string(),
            unit: "kg".to_string(),
            quantity: create_decimal("10.5"),
            price: create_decimal("9.50"),
            original_price: create_decimal("10.00"),
            original_amount: create_decimal("105.00"),
            discount_rate: create_decimal("0.05"),
            total: create_decimal("99.75"),
            remark: Some("Fresh apples".to_string()),
            processing_services: vec![create_test_processing_service()],
        }
    }

    // Tests for ReceiptOperationType enum
    mod receipt_operation_type_tests {
        use super::*;

        #[test]
        fn test_receipt_operation_type_from_string() {
            assert_eq!(
                ReceiptOperationType::try_from("SIGN".to_string()).unwrap(),
                ReceiptOperationType::Sign
            );
            assert_eq!(
                ReceiptOperationType::try_from("RETURN".to_string()).unwrap(),
                ReceiptOperationType::Return
            );
            assert_eq!(
                ReceiptOperationType::try_from("EXCHANGE".to_string()).unwrap(),
                ReceiptOperationType::Exchange
            );
            assert!(ReceiptOperationType::try_from("UNKNOWN".to_string()).is_err());
        }

        #[test]
        fn test_receipt_operation_type_to_string() {
            assert_eq!(String::from(ReceiptOperationType::Sign), "SIGN");
            assert_eq!(String::from(ReceiptOperationType::Return), "RETURN");
            assert_eq!(String::from(ReceiptOperationType::Exchange), "EXCHANGE");
        }

        #[test]
        fn test_receipt_operation_type_serialization() {
            let receipt = OrderReceipt {
                order_detail_id: 1,
                order_code: VALID_ORDER_CODE.to_string(),
                product_code: VALID_PRODUCT_CODE.to_string(),
                product_name: "Apple".to_string(),
                operation_type: ReceiptOperationType::Sign,
                quantity: create_decimal("10.0"),
                reason: "正常签收".to_string(),
                unit: "kg".to_string(),
                evidence_images: None,
            };

            let json = serde_json::to_value(&receipt).unwrap();
            assert_eq!(json["operationType"], "SIGN");

            // Test deserialization
            let json_data = json!({
                "id": 1,
                "orderId": "ORD001",
                "productId": "P001",
                "productName": "Apple",
                "operationType": "RETURN",
                "quantity": "5.0",
                "reason": "质量问题",
                "unit": "kg"
            });

            let deserialized: OrderReceipt = serde_json::from_value(json_data).unwrap();
            assert_eq!(deserialized.operation_type, ReceiptOperationType::Return);
        }
    }

    // Tests for CreateOrderDTO and related structures
    mod create_order_dto_tests {
        use super::*;

        #[test]
        fn test_create_order_dto_serialization() {
            let order_dto = CreateOrderDTO {
                total_amount: create_decimal(VALID_TOTAL_AMOUNT),
                delivery_info: create_test_delivery_info(),
                items: vec![create_test_order_item()],
            };

            let json = serde_json::to_value(&order_dto).unwrap();

            // Test field renaming
            assert!(json.get("totalAmount").is_some());
            assert!(json.get("deliveryInfo").is_some());
            assert!(json.get("items").is_some());
            assert!(json.get("total_amount").is_none());
            assert!(json.get("delivery_info").is_none());
        }

        #[test]
        fn test_delivery_info_field_mapping() {
            let delivery_info = create_test_delivery_info();
            let json = serde_json::to_value(&delivery_info).unwrap();

            // Test all field mappings
            assert!(json.get("deliveryDate").is_some());
            assert!(json.get("address").is_some());
            assert!(json.get("contactName").is_some());
            assert!(json.get("contactPhone").is_some());

            // Test original field names are not present
            assert!(json.get("delivery_date").is_none());
            assert!(json.get("delivery_address").is_none());
            assert!(json.get("contact_name").is_none());
            assert!(json.get("contact_phone").is_none());
        }

        #[test]
        fn test_create_order_item_field_mapping() {
            let item = create_test_order_item();
            let json = serde_json::to_value(&item).unwrap();

            // Test all renamed fields
            let expected_mappings = vec![
                ("productId", "product_code"),
                ("name", "product_name"),
                ("categoryId", "category_id"),
                ("category", "category_name"),
                ("quantity", "quantity"),
                ("price", "price"),
                ("originalPrice", "original_price"),
                ("originalTotal", "original_amount"),
                ("discountRate", "discount_rate"),
                ("total", "total"),
                ("customNote", "remark"),
                ("processingServices", "processing_services"),
            ];

            for (json_field, _rust_field) in expected_mappings {
                assert!(json.get(json_field).is_some(), "Missing field: {}", json_field);
            }
        }

        #[test]
        fn test_processing_service_field_mapping() {
            let service = create_test_processing_service();
            let json = serde_json::to_value(&service).unwrap();

            assert!(json.get("type").is_some());
            assert!(json.get("description").is_some());
            assert_eq!(json["type"], "切片");
        }

        #[test]
        fn test_decimal_precision_handling() {
            let item = CreateOrderItem {
                product_code: VALID_PRODUCT_CODE.to_string(),
                product_name: "Test".to_string(),
                category_id: 1,
                category_name: "Test".to_string(),
                unit: "kg".to_string(),
                quantity: create_decimal("10.123456"),
                price: create_decimal("9.99"),
                original_price: create_decimal("10.00"),
                original_amount: create_decimal("101.23456"),
                discount_rate: create_decimal("0.01"),
                total: create_decimal("100.22226"),
                remark: None,
                processing_services: vec![],
            };

            let json = serde_json::to_string(&item).unwrap();
            let deserialized: CreateOrderItem = serde_json::from_str(&json).unwrap();

            assert_eq!(item.quantity, deserialized.quantity);
            assert_eq!(item.price, deserialized.price);
            assert_eq!(item.total, deserialized.total);
        }
    }

    // Tests for OrderResponse and query DTOs
    mod order_response_tests {
        use super::*;

        #[test]
        fn test_order_response_serialization() {
            let response = OrderResponse {
                order_code: VALID_ORDER_CODE.to_string(),
                total_amount: create_decimal("150.75"),
                actual_amount: create_decimal("143.21"),
                delivery_date: chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                delivery_address: "Test Address".to_string(),
                order_status: "CONFIRMED".to_string(),
                created_at: Some(Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap()),
                after_sale_at: None,
            };

            let json = serde_json::to_value(&response).unwrap();

            // Test field mappings
            assert!(json.get("orderCode").is_some());
            assert!(json.get("totalAmount").is_some());
            assert!(json.get("actualAmount").is_some());
            assert!(json.get("deliveryDate").is_some());
            assert!(json.get("deliveryAddress").is_some());
            assert!(json.get("orderStatus").is_some());
            assert!(json.get("createdAt").is_some());
            assert!(json.get("afterSaleAt").is_some());

            assert_eq!(json["orderCode"], VALID_ORDER_CODE);
            assert_eq!(json["orderStatus"], "CONFIRMED");
        }

        #[test]
        fn test_order_query_params_optional_fields() {
            // Test with all fields present
            let params_full = OrderQueryParams {
                page: Some(1),
                page_size: Some(20),
                order_status: Some("PENDING".to_string()),
            };

            let json = serde_json::to_value(&params_full).unwrap();
            assert_eq!(json["page"], 1);
            assert_eq!(json["pageSize"], 20);
            assert_eq!(json["orderStatus"], "PENDING");

            // Test with no fields
            let params_empty = OrderQueryParams {
                page: None,
                page_size: None,
                order_status: None,
            };

            let json = serde_json::to_value(&params_empty).unwrap();
            assert!(json["page"].is_null());
            assert!(json["pageSize"].is_null());
            assert!(json["orderStatus"].is_null());
        }
    }

    // Tests for complex order structures
    mod order_detail_tests {
        use super::*;

        #[test]
        fn test_order_item_comprehensive_mapping() {
            let order_item = OrderItem {
                id: 1,
                order_code: VALID_ORDER_CODE.to_string(),
                customer_name: "Customer A".to_string(),
                order_status: "PROCESSING".to_string(),
                total_amount: create_decimal("200.00"),
                discount_amount: create_decimal("20.00"),
                actual_amount: create_decimal("180.00"),
                delivery_date: chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                delivery_address: "123 Main St".to_string(),
                contact_name: "John Doe".to_string(),
                contact_phone: VALID_PHONE.to_string(),
                remark: Some("Special handling".to_string()),
                created_by: "admin".to_string(),
                created_at: Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
                confirmed_by: Some("manager".to_string()),
                confirmed_at: Some(Utc.with_ymd_and_hms(2024, 1, 2, 10, 0, 0).unwrap()),
                processed_by: None,
                processed_at: None,
                stocked_by: None,
                stocked_at: None,
                after_sale_at: None,
                rejected_by: None,
                rejected_at: None,
                reject_reason: None,
                completed_by: None,
                delivery_staff_name: None,
                delivery_staff_phone: None,
                provider_name: Some("Provider A".to_string()),
                completed_at: None,
            };

            let json = serde_json::to_value(&order_item).unwrap();

            // Test key field mappings
            assert_eq!(json["orderCode"], VALID_ORDER_CODE);
            assert_eq!(json["customerName"], "Customer A");
            assert_eq!(json["orderStatus"], "PROCESSING");
            assert_eq!(json["totalAmount"], "200.00");
            assert_eq!(json["deliveryAddress"], "123 Main St");
            assert_eq!(json["contactName"], "John Doe");
            assert_eq!(json["createdBy"], "admin");
            assert_eq!(json["confirmBy"], "manager");
            assert_eq!(json["providerName"], "Provider A");

            // Test optional fields
            assert!(json["processedBy"].is_null());
            assert!(json["completedAt"].is_null());
        }

        #[test]
        fn test_order_detail_response_structure() {
            let order = OrderItem {
                id: 1,
                order_code: VALID_ORDER_CODE.to_string(),
                customer_name: "Test Customer".to_string(),
                order_status: "CONFIRMED".to_string(),
                total_amount: create_decimal("100.00"),
                discount_amount: create_decimal("5.00"),
                actual_amount: create_decimal("95.00"),
                delivery_date: chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                delivery_address: "Test Address".to_string(),
                contact_name: "Test Contact".to_string(),
                contact_phone: VALID_PHONE.to_string(),
                remark: None,
                created_by: "system".to_string(),
                created_at: Utc::now(),
                confirmed_by: None,
                confirmed_at: None,
                processed_by: None,
                processed_at: None,
                stocked_by: None,
                stocked_at: None,
                after_sale_at: None,
                rejected_by: None,
                rejected_at: None,
                reject_reason: None,
                completed_by: None,
                delivery_staff_name: None,
                delivery_staff_phone: None,
                provider_name: None,
                completed_at: None,
            };

            let order_detail = OrderDetail {
                id: 1,
                product_code: VALID_PRODUCT_CODE.to_string(),
                product_name: "Apple".to_string(),
                category_id: VALID_CATEGORY_ID,
                category_name: "Fruits".to_string(),
                unit: "kg".to_string(),
                quantity: create_decimal("10.0"),
                original_price: create_decimal("10.00"),
                discount_rate: create_decimal("0.05"),
                actual_price: create_decimal("9.50"),
                actual_quantity: Some(create_decimal("9.8")),
                actual_amount: Some(create_decimal("93.10")),
                total_amount: create_decimal("95.00"),
                processing_requirements: Some("切片".to_string()),
                remark: None,
                status: Some("CONFIRMED".to_string()),
            };

            let receipt = OrderReceipt {
                order_detail_id: 1,
                order_code: VALID_ORDER_CODE.to_string(),
                product_code: VALID_PRODUCT_CODE.to_string(),
                product_name: "Apple".to_string(),
                operation_type: ReceiptOperationType::Sign,
                quantity: create_decimal("9.8"),
                reason: "正常签收".to_string(),
                unit: "kg".to_string(),
                evidence_images: None,
            };

            let response = OrderDetailResponse {
                order,
                items: vec![order_detail],
                receipts: vec![receipt],
            };

            let json = serde_json::to_value(&response).unwrap();
            
            assert!(json.get("order").is_some());
            assert!(json.get("items").is_some());
            assert!(json.get("receipts").is_some());
            assert!(json["items"].as_array().unwrap().len() == 1);
            assert!(json["receipts"].as_array().unwrap().len() == 1);
        }
    }

    // Tests for OrderStatus business logic
    mod order_status_business_logic_tests {
        use super::*;

        #[test]
        fn test_order_status_from_str() {
            // Test valid statuses
            assert_eq!(OrderStatus::from_str("PENDING").unwrap(), OrderStatus::Pending);
            assert_eq!(OrderStatus::from_str("CONFIRMED").unwrap(), OrderStatus::Confirmed);
            assert_eq!(OrderStatus::from_str("PROCESSING").unwrap(), OrderStatus::Processing);
            assert_eq!(OrderStatus::from_str("STOCKED").unwrap(), OrderStatus::Stocked);
            assert_eq!(OrderStatus::from_str("AFTER_SALE").unwrap(), OrderStatus::AfterSale);
            assert_eq!(OrderStatus::from_str("REJECTED").unwrap(), OrderStatus::Rejected);
            assert_eq!(OrderStatus::from_str("COMPLETED").unwrap(), OrderStatus::Completed);

            // Test invalid status
            assert!(OrderStatus::from_str("INVALID").is_err());
            assert!(OrderStatus::from_str("").is_err());
            assert!(OrderStatus::from_str("pending").is_err()); // Case sensitive
        }

        #[test]
        fn test_order_status_valid_transitions() {
            // Test all valid transitions
            assert!(OrderStatus::validate_transition("PENDING", "CONFIRMED").is_ok());
            assert!(OrderStatus::validate_transition("CONFIRMED", "PROCESSING").is_ok());
            assert!(OrderStatus::validate_transition("PROCESSING", "STOCKED").is_ok());
            assert!(OrderStatus::validate_transition("STOCKED", "COMPLETED").is_ok());
            assert!(OrderStatus::validate_transition("STOCKED", "AFTER_SALE").is_ok());
            assert!(OrderStatus::validate_transition("AFTER_SALE", "COMPLETED").is_ok());
            assert!(OrderStatus::validate_transition("AFTER_SALE", "REJECTED").is_ok());
            assert!(OrderStatus::validate_transition("REJECTED", "COMPLETED").is_ok());
        }

        #[test]
        fn test_order_status_invalid_transitions() {
            // Test some key invalid transitions
            assert!(OrderStatus::validate_transition("PENDING", "PROCESSING").is_err());
            assert!(OrderStatus::validate_transition("PENDING", "COMPLETED").is_err());
            assert!(OrderStatus::validate_transition("CONFIRMED", "COMPLETED").is_err());
            assert!(OrderStatus::validate_transition("PROCESSING", "COMPLETED").is_err());
            assert!(OrderStatus::validate_transition("COMPLETED", "PENDING").is_err());
            assert!(OrderStatus::validate_transition("COMPLETED", "REJECTED").is_err());

            // Test invalid status strings
            assert!(OrderStatus::validate_transition("INVALID", "CONFIRMED").is_err());
            assert!(OrderStatus::validate_transition("PENDING", "INVALID").is_err());
        }

        #[test]
        fn test_tenant_permission_validation() {
            use crate::repositories::TenantType;

            // Test valid permissions
            assert!(OrderStatus::validate_transition_with_tenant(
                "PENDING", "CONFIRMED", &TenantType::Market.to_string()
            ).is_ok());

            assert!(OrderStatus::validate_transition_with_tenant(
                "CONFIRMED", "PROCESSING", &TenantType::Provider.to_string()
            ).is_ok());

            assert!(OrderStatus::validate_transition_with_tenant(
                "STOCKED", "COMPLETED", &TenantType::Customer.to_string()
            ).is_ok());

            // Test invalid permissions
            assert!(OrderStatus::validate_transition_with_tenant(
                "PENDING", "CONFIRMED", &TenantType::Customer.to_string()
            ).is_err());

            assert!(OrderStatus::validate_transition_with_tenant(
                "CONFIRMED", "PROCESSING", &TenantType::Market.to_string()
            ).is_err());

            // Test invalid tenant type
            assert!(OrderStatus::validate_transition_with_tenant(
                "PENDING", "CONFIRMED", "INVALID_TENANT"
            ).is_err());
        }

        #[test]
        fn test_complex_workflow_scenarios() {
            use crate::repositories::TenantType;

            // Test complete successful workflow
            let market = TenantType::Market.to_string();
            let provider = TenantType::Provider.to_string();
            let customer = TenantType::Customer.to_string();

            // Happy path: PENDING -> CONFIRMED -> PROCESSING -> STOCKED -> COMPLETED
            assert!(OrderStatus::validate_transition_with_tenant("PENDING", "CONFIRMED", &market).is_ok());
            assert!(OrderStatus::validate_transition_with_tenant("CONFIRMED", "PROCESSING", &provider).is_ok());
            assert!(OrderStatus::validate_transition_with_tenant("PROCESSING", "STOCKED", &provider).is_ok());
            assert!(OrderStatus::validate_transition_with_tenant("STOCKED", "COMPLETED", &customer).is_ok());

            // After-sale scenario: STOCKED -> AFTER_SALE -> COMPLETED
            assert!(OrderStatus::validate_transition_with_tenant("STOCKED", "AFTER_SALE", &customer).is_ok());
            assert!(OrderStatus::validate_transition_with_tenant("AFTER_SALE", "COMPLETED", &customer).is_ok());

            // Rejection scenario: STOCKED -> AFTER_SALE -> REJECTED -> COMPLETED
            assert!(OrderStatus::validate_transition_with_tenant("AFTER_SALE", "REJECTED", &customer).is_ok());
            assert!(OrderStatus::validate_transition_with_tenant("REJECTED", "COMPLETED", &customer).is_ok());
        }
    }

    // Tests for utility DTOs and edge cases
    mod utility_dto_tests {
        use super::*;

        #[test]
        fn test_dispatch_order_dto() {
            let dispatch_dto = DispatchOrderDTO {
                provider_id: 123,
                confirmed_by: "manager".to_string(),
            };

            let json = serde_json::to_value(&dispatch_dto).unwrap();
            assert_eq!(json["providerId"], 123);
            assert_eq!(json["confirmedBy"], "manager");
        }

        #[test]
        fn test_actual_quantity_dto() {
            let actual_qty = ActualQuantity {
                id: 1,
                actual_quantity: create_decimal("9.5"),
            };

            let dto = ActualQuantityDTO::new(
                "warehouse_staff".to_string(),
                vec![actual_qty],
            );

            let json = serde_json::to_value(&dto).unwrap();
            assert_eq!(json["stockedBy"], "warehouse_staff");
            assert!(json["items"].as_array().unwrap().len() == 1);
            assert_eq!(json["items"][0]["actualQuantity"], "9.5");
        }

        #[test]
        fn test_update_order_status_dto() {
            let update_dto = UpdateOrderStatusDTO {
                operate_by: "operator".to_string(),
                status: "CONFIRMED".to_string(),
            };

            let json = serde_json::to_value(&update_dto).unwrap();
            assert_eq!(json["operateBy"], "operator");
            assert_eq!(json["status"], "CONFIRMED");
        }

        #[test]
        fn test_products_summary_with_orders_dto() {
            let summary = ProductsSummaryWithOrdersDTO {
                product_code: VALID_PRODUCT_CODE.to_string(),
                product_name: "Apple".to_string(),
                total_quantity: Some(create_decimal("100.5")),
                unit: "kg".to_string(),
                processing_requirements: Some("切片".to_string()),
                customer_name: Some("Customer A".to_string()),
                remark: None,
            };

            let json = serde_json::to_value(&summary).unwrap();
            assert_eq!(json["productId"], VALID_PRODUCT_CODE);
            assert_eq!(json["productName"], "Apple");
            assert_eq!(json["totalQuantity"], "100.5");
            assert_eq!(json["processingRequirements"], "切片");
            assert_eq!(json["customerName"], "Customer A");
        }
    }

    // Integration tests
    mod integration_tests {
        use super::*;

        #[test]
        fn test_complete_order_lifecycle_serialization() {
            // Test creating an order
            let create_order = CreateOrderDTO {
                total_amount: create_decimal("200.50"),
                delivery_info: create_test_delivery_info(),
                items: vec![create_test_order_item()],
            };

            let create_json = serde_json::to_string(&create_order).unwrap();
            let _deserialized_create: CreateOrderDTO = serde_json::from_str(&create_json).unwrap();

            // Test order response
            let order_response = OrderResponse {
                order_code: VALID_ORDER_CODE.to_string(),
                total_amount: create_decimal("200.50"),
                actual_amount: create_decimal("190.48"),
                delivery_date: chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                delivery_address: "Test Address".to_string(),
                order_status: "CONFIRMED".to_string(),
                created_at: Some(Utc::now()),
                after_sale_at: None,
            };

            let response_json = serde_json::to_string(&order_response).unwrap();
            let _deserialized_response: OrderResponse = serde_json::from_str(&response_json).unwrap();

            // Test order receipt
            let receipt = OrderReceipt {
                order_detail_id: 1,
                order_code: VALID_ORDER_CODE.to_string(),
                product_code: VALID_PRODUCT_CODE.to_string(),
                product_name: "Apple".to_string(),
                operation_type: ReceiptOperationType::Sign,
                quantity: create_decimal("10.0"),
                reason: "正常签收".to_string(),
                unit: "kg".to_string(),
                evidence_images: Some("image1.jpg,image2.jpg".to_string()),
            };

            let receipt_json = serde_json::to_string(&receipt).unwrap();
            let deserialized_receipt: OrderReceipt = serde_json::from_str(&receipt_json).unwrap();

            assert_eq!(receipt.operation_type, deserialized_receipt.operation_type);
            assert_eq!(receipt.evidence_images, deserialized_receipt.evidence_images);
        }

        #[test]
        fn test_edge_cases_and_optional_fields() {
            // Test with all optional fields as None
            let minimal_order_item = OrderItem {
                id: 1,
                order_code: VALID_ORDER_CODE.to_string(),
                customer_name: "Test".to_string(),
                order_status: "PENDING".to_string(),
                total_amount: create_decimal("0.00"),
                discount_amount: create_decimal("0.00"),
                actual_amount: create_decimal("0.00"),
                delivery_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                delivery_address: "".to_string(),
                contact_name: "".to_string(),
                contact_phone: "".to_string(),
                remark: None,
                created_by: "system".to_string(),
                created_at: Utc::now(),
                confirmed_by: None,
                confirmed_at: None,
                processed_by: None,
                processed_at: None,
                stocked_by: None,
                stocked_at: None,
                after_sale_at: None,
                rejected_by: None,
                rejected_at: None,
                reject_reason: None,
                completed_by: None,
                delivery_staff_name: None,
                delivery_staff_phone: None,
                provider_name: None,
                completed_at: None,
            };

            let json = serde_json::to_string(&minimal_order_item).unwrap();
            let _deserialized: OrderItem = serde_json::from_str(&json).unwrap();

            // Test OrderReceipt without evidence images
            let receipt_no_images = OrderReceipt {
                order_detail_id: 1,
                order_code: VALID_ORDER_CODE.to_string(),
                product_code: VALID_PRODUCT_CODE.to_string(),
                product_name: "Test Product".to_string(),
                operation_type: ReceiptOperationType::Return,
                quantity: create_decimal("1.0"),
                reason: "测试原因".to_string(),
                unit: "pcs".to_string(),
                evidence_images: None,
            };

            let json = serde_json::to_value(&receipt_no_images).unwrap();
            // Should not have evidenceImages field when None
            assert!(json.get("evidenceImages").is_none());
        }

        #[test]
        fn test_new_optimization_methods() {
            // Test CreateOrderDTO optimization methods
            let delivery_info = DeliveryInfo::new(
                "2024-12-31".to_string(),
                "123 Main St".to_string(),
                "John Doe".to_string(),
                "13800138000".to_string(),
            );
            assert!(delivery_info.is_complete());

            let order_item = create_test_order_item();
            let create_order = CreateOrderDTO::new(
                create_decimal("200.50"),
                delivery_info,
                vec![order_item.clone()],
            );

            assert_eq!(create_order.items_count(), 1);
            assert!(!create_order.is_empty());
            assert_eq!(create_order.total_quantity(), create_decimal("10.5"));

            // Test CreateOrderItem optimization methods
            assert!(order_item.has_discount());
            assert!(order_item.has_processing_services());
            assert_eq!(order_item.processing_services_count(), 1);
            assert_eq!(order_item.discount_amount(), create_decimal("5.25"));

            // Test ProcessingService
            let service = ProcessingService::simple("切片".to_string());
            assert_eq!(service.name, "切片");
            assert!(service.description.is_none());

            // Test OrderQueryParams with builder pattern
            let query = OrderQueryParams::new()
                .with_page(2)
                .with_page_size(50)
                .with_status("CONFIRMED".to_string());

            assert_eq!(query.effective_page(), 2);
            assert_eq!(query.effective_page_size(), 50);
            assert_eq!(query.order_status, Some("CONFIRMED".to_string()));

            // Test OrderResponse optimization methods
            let order_response = OrderResponse {
                order_code: "ORD123".to_string(),
                total_amount: create_decimal("100.00"),
                actual_amount: create_decimal("90.00"),
                delivery_date: chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                delivery_address: "Test Address".to_string(),
                order_status: "CONFIRMED".to_string(),
                created_at: Some(Utc::now()),
                after_sale_at: None,
            };

            assert!(order_response.has_discount());
            assert_eq!(order_response.discount_amount(), create_decimal("10.00"));
            assert!(!order_response.is_after_sale());
            assert!(order_response.status_enum().is_ok());

            // Test OrderReceipt optimization methods
            let receipt = OrderReceipt::new(
                1,
                "ORD123".to_string(),
                "P001".to_string(),
                "Test Product".to_string(),
                ReceiptOperationType::Sign,
                create_decimal("10.0"),
                "正常签收".to_string(),
                "kg".to_string(),
                Some("image1.jpg,image2.jpg".to_string()),
            );

            assert!(receipt.has_evidence_images());
            assert!(receipt.is_sign_receipt());
            assert!(!receipt.is_return_receipt());
            assert!(!receipt.is_exchange_receipt());
            assert_eq!(receipt.evidence_images_list(), vec!["image1.jpg", "image2.jpg"]);

            // Test ActualQuantityDTO
            let actual_qty = ActualQuantity {
                id: 1,
                actual_quantity: create_decimal("9.5"),
            };
            let actual_qty_dto = ActualQuantityDTO::new(
                "warehouse_staff".to_string(),
                vec![actual_qty],
            );

            assert!(actual_qty_dto.has_items());
            assert_eq!(actual_qty_dto.items_count(), 1);
            assert_eq!(actual_qty_dto.total_actual_quantity(), create_decimal("9.5"));
        }
    }
}
