use chrono::{DateTime, Utc};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq)]
pub(crate) struct FinancialResponseDto {
    pub(crate) id: Option<i32>,

    #[serde(rename = "tenantId")]
    pub(crate) tenant_id: Option<i32>,

    #[serde(rename = "creditScore")]
    pub(crate) credit_score: Option<Decimal>,

    #[serde(rename = "creditLimit")]
    pub(crate) credit_limit: Option<Decimal>,

    #[serde(rename = "depositAmount")]
    pub(crate) deposit_amount: Option<Decimal>,

    #[serde(rename = "paymentPeriodDays")]
    pub(crate) payment_period_days: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "createdAt")]
    pub(crate) created_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedAt")]
    pub(crate) updated_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "deletedAt")]
    pub(crate) deleted_at: Option<DateTime<Utc>>,
}
