use crate::common::AppError;
use axum::{
    extract::{rejection::JsonRejection, FromRequest, Request},
    Json,
};
use lazy_static::lazy_static;
use regex::Regex;
use serde::de::DeserializeOwned;
use validator::Validate;

pub(crate) mod auth;
pub(crate) mod category;
pub(crate) mod delivery_staff;
pub(crate) mod discount;
pub(crate) mod financial;
pub(crate) mod order;
pub(crate) mod price;
pub(crate) mod products;
pub(crate) mod reconciliation_statement;
pub(crate) mod system_log;
pub(crate) mod tenants;
pub(crate) mod users;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ValidatedJSON<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJSON<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJSON(value))
    }
}

lazy_static! {
    pub(crate) static ref PHONE_REGEX: Regex = Regex::new(r"^1[3-9]\d{9}$").unwrap();
}

pub(crate) fn validate_phone(phone: &str) -> Result<(), validator::ValidationError> {
    if !PHONE_REGEX.is_match(phone) {
        return Err(validator::ValidationError::new("Invalid phone number"));
    }
    Ok(())
}

#[test]
fn test_phone_regex() {
    // 有效的 11 位手机号
    assert!(PHONE_REGEX.is_match("13812345678")); // 正常手机号
    assert!(PHONE_REGEX.is_match("19999999999")); // 新号段
    assert!(PHONE_REGEX.is_match("15712345678")); // 移动联通号段

    // 无效手机号（边界情况）
    assert!(!PHONE_REGEX.is_match("12812345678")); // 第二位不是 3-9
    assert!(!PHONE_REGEX.is_match("1381234567")); // 10 位，不足
    assert!(!PHONE_REGEX.is_match("138123456789")); // 12 位，超出
    assert!(!PHONE_REGEX.is_match("23812345678")); // 非1开头
    assert!(!PHONE_REGEX.is_match("1a812345678")); // 包含字母
    assert!(!PHONE_REGEX.is_match("")); // 空字符串
    assert!(!PHONE_REGEX.is_match("138-1234-5678")); // 带分隔符
    assert!(!PHONE_REGEX.is_match(" 13812345678 ")); // 带空格
}
