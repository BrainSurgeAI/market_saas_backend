use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ReconciliationStatementDTO {
    pub id: Option<i32>,

    #[serde(rename = "statementCode")]
    pub statement_code: String,

    #[serde(rename = "customerId")]
    pub customer_id: i32,

    #[serde(rename = "marketId")]
    pub market_id: i32,

    #[serde(rename = "providerId")]
    pub provider_id: i32,

    #[serde(rename = "supplierName")]
    pub supplier_name: Option<String>,

    #[serde(rename = "customerName")]
    pub customer_name: Option<String>,

    #[serde(rename = "startDate")]
    pub start_date: NaiveDate,

    #[serde(rename = "endDate")]
    pub end_date: NaiveDate,

    #[serde(rename = "totalAmount")]
    pub total_amount: Decimal,

    #[serde(rename = "discountAmount")]
    pub discount_amount: Decimal,

    #[serde(rename = "actualAmount")]
    pub actual_amount: Decimal,

    pub status: String,

    pub remark: Option<String>,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "confirmedBy")]
    pub confirmed_by: Option<String>,

    #[serde(rename = "confirmedAt")]
    pub confirmed_at: Option<DateTime<Utc>>,

    #[serde(rename = "completedBy")]
    pub completed_by: Option<String>,

    #[serde(rename = "completedAt")]
    pub completed_at: Option<DateTime<Utc>>,

    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ReconciliationStatementOrderDTO {
    pub id: Option<i32>,

    #[serde(rename = "statementId")]
    pub statement_id: i32,

    #[serde(rename = "orderId")]
    pub order_id: i32,

    #[serde(rename = "orderCode")]
    pub order_code: String,

    #[serde(rename = "orderDate")]
    pub order_date: NaiveDate,

    #[serde(rename = "totalAmount")]
    pub total_amount: Decimal,

    #[serde(rename = "actualAmount")]
    pub actual_amount: Decimal,

    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
}



// #[derive(Debug, Serialize, Deserialize, FromRow)]
// pub struct ReconciliationStatementDetailDTO {
//     pub id: Option<i32>,

//     #[serde(rename = "statementId")]
//     pub statement_id: i32,

//     #[serde(rename = "orderDetailId")]
//     pub order_detail_id: i32,

//     #[serde(rename = "productCode")]
//     pub product_code: String,

//     #[serde(rename = "productName")]
//     pub product_name: String,

//     #[serde(rename = "categoryName")]
//     pub category_name: String,

//     pub unit: String,

//     #[serde(rename = "originalQuantity")]
//     pub original_quantity: Decimal,

//     #[serde(rename = "actualQuantity")]
//     pub actual_quantity: Decimal,

//     #[serde(rename = "receiptQuantity")]
//     pub receipt_quantity: Decimal,

//     #[serde(rename = "returnedQuantity")]
//     pub returned_quantity: Decimal,

//     pub price: Decimal,

//     #[serde(rename = "originalAmount")]
//     pub original_amount: Decimal,

//     #[serde(rename = "actualAmount")]
//     pub actual_amount: Decimal,

//     #[serde(rename = "orderDate")]
//     pub order_date: NaiveDate,

//     #[serde(rename = "createdAt")]
//     pub created_at: Option<DateTime<Utc>>,
// }

// /// 用于 API 响应的完整对账单信息，包含关联的订单列表
// #[derive(Debug, Serialize, Deserialize)]
// pub struct ReconciliationStatementResponseDTO {
//     pub id: Option<i32>,

//     #[serde(rename = "statementCode")]
//     pub statement_code: String,

//     #[serde(rename = "customerId")]
//     pub customer_id: i32,

//     #[serde(rename = "marketId")]
//     pub market_id: i32,

//     #[serde(rename = "providerId")]
//     pub provider_id: i32,

//     #[serde(rename = "supplierName")]
//     pub supplier_name: Option<String>,

//     #[serde(rename = "startDate")]
//     pub start_date: NaiveDate,

//     #[serde(rename = "endDate")]
//     pub end_date: NaiveDate,

//     #[serde(rename = "totalAmount")]
//     pub total_amount: Decimal,

//     #[serde(rename = "discountAmount")]
//     pub discount_amount: Decimal,

//     #[serde(rename = "actualAmount")]
//     pub actual_amount: Decimal,

//     pub status: String,

//     pub remark: Option<String>,

//     #[serde(rename = "createdBy")]
//     pub created_by: String,

//     #[serde(rename = "confirmedBy")]
//     pub confirmed_by: Option<String>,

//     #[serde(rename = "confirmedAt")]
//     pub confirmed_at: Option<DateTime<Utc>>,

//     #[serde(rename = "completedBy")]
//     pub completed_by: Option<String>,

//     #[serde(rename = "completedAt")]
//     pub completed_at: Option<DateTime<Utc>>,

//     #[serde(rename = "createdAt")]
//     pub created_at: Option<DateTime<Utc>>,

//     #[serde(rename = "updatedAt")]
//     pub updated_at: Option<DateTime<Utc>>,

//     /// 关联的订单列表
//     pub orders: Vec<ReconciliationStatementOrderDTO>,
// }

// /// 用于 API 响应的完整对账单订单信息，包含订单详情
// #[derive(Debug, Serialize, Deserialize)]
// pub struct ReconciliationStatementOrderResponseDTO {
//     pub id: Option<i32>,

//     #[serde(rename = "statementId")]
//     pub statement_id: i32,

//     #[serde(rename = "orderId")]
//     pub order_id: i32,

//     #[serde(rename = "orderCode")]
//     pub order_code: String,

//     #[serde(rename = "orderDate")]
//     pub order_date: NaiveDate,

//     #[serde(rename = "totalAmount")]
//     pub total_amount: Decimal,

//     #[serde(rename = "actualAmount")]
//     pub actual_amount: Decimal,

//     #[serde(rename = "createdAt")]
//     pub created_at: Option<DateTime<Utc>>,

//     /// 订单明细列表
//     pub details: Vec<ReconciliationStatementDetailDTO>,
// }
