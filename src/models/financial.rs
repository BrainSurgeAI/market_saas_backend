use chrono::{DateTime, Utc};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow, Validate, Clone, PartialEq, Eq)]
pub struct Financial {
    pub id: Option<i32>,

    #[serde(rename = "tenantId")]
    pub tenant_id: Option<i32>,

    #[serde(rename = "creditScore")]
    pub credit_score: Option<Decimal>,

    #[serde(rename = "creditLimit")]
    pub credit_limit: Option<Decimal>,

    #[serde(rename = "depositAmount")]
    pub deposit_amount: Option<Decimal>,

    #[serde(rename = "paymentPeriodDays")]
    pub payment_period_days: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}
