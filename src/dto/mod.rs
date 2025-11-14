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

use chrono::{NaiveDate, Utc};

use tracing::error;

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

pub(super) fn validate_delivery_date(delivery_date: &str) -> Result<(), validator::ValidationError> {
    let parsed_date = NaiveDate::parse_from_str(delivery_date, "%Y-%m-%d").map_err(|e| {
        error!("配送日期格式错误: {:#?}", e);
        let mut error = validator::ValidationError::new("invalid_delivery_date_format");
        error.message = Some(format!("配送日期格式错误: {}", delivery_date).into());
        error
    })?;

    // 检查配送日期不能早于今天
    let today = Utc::now().date_naive();
    if parsed_date < today {
        let mut error = validator::ValidationError::new("delivery_date_too_early");
        error.message = Some(format!("配送日期不能早于今天，配送日期: {}", parsed_date).into());
        return Err(error);
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

#[cfg(test)]
mod delivery_date_tests {
    use super::validate_delivery_date;
    use chrono::{Duration, NaiveDate, Utc};

    #[test]
    fn test_valid_delivery_date_today() {
        // 测试今天的日期（应该通过验证）
        let today = Utc::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();
        assert!(validate_delivery_date(&today_str).is_ok());
    }

    #[test]
    fn test_valid_delivery_date_tomorrow() {
        // 测试明天的日期（应该通过验证）
        let tomorrow = Utc::now().date_naive() + Duration::days(1);
        let tomorrow_str = tomorrow.format("%Y-%m-%d").to_string();
        assert!(validate_delivery_date(&tomorrow_str).is_ok());
    }

    #[test]
    fn test_valid_delivery_date_future() {
        // 测试未来的日期（应该通过验证）
        let future_date = Utc::now().date_naive() + Duration::days(30);
        let future_str = future_date.format("%Y-%m-%d").to_string();
        assert!(validate_delivery_date(&future_str).is_ok());
    }

    #[test]
    fn test_invalid_delivery_date_yesterday() {
        // 测试昨天的日期（应该失败）
        let yesterday = Utc::now().date_naive() - Duration::days(1);
        let yesterday_str = yesterday.format("%Y-%m-%d").to_string();
        let result = validate_delivery_date(&yesterday_str);
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "delivery_date_too_early");
            assert!(error.message.is_some());
            let msg = error.message.unwrap();
            assert!(msg.contains("配送日期不能早于今天"));
        }
    }

    #[test]
    fn test_invalid_delivery_date_past() {
        // 测试过去的日期（应该失败）
        let past_date = Utc::now().date_naive() - Duration::days(365);
        let past_str = past_date.format("%Y-%m-%d").to_string();
        let result = validate_delivery_date(&past_str);
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "delivery_date_too_early");
        }
    }

    #[test]
    fn test_invalid_date_format_empty_string() {
        // 测试空字符串（应该失败）
        let result = validate_delivery_date("");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
            assert!(error.message.is_some());
            let msg = error.message.unwrap();
            assert!(msg.contains("配送日期格式错误"));
        }
    }

    #[test]
    fn test_invalid_date_format_wrong_separator() {
        // 测试错误的分隔符（应该失败）
        let result = validate_delivery_date("2024/01/01");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
        }
    }

    #[test]
    fn test_invalid_date_format_no_separator() {
        // 测试没有分隔符（应该失败）
        let result = validate_delivery_date("20240101");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
        }
    }

    #[test]
    fn test_invalid_date_format_wrong_order() {
        // 测试错误的日期顺序（应该失败）
        let result = validate_delivery_date("01-01-2024");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
        }
    }

    #[test]
    fn test_invalid_date_format_invalid_month() {
        // 测试无效的月份（应该失败）
        let result = validate_delivery_date("2024-13-01");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
        }
    }

    #[test]
    fn test_invalid_date_format_invalid_day() {
        // 测试无效的日期（应该失败）
        let result = validate_delivery_date("2024-02-30");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
        }
    }

    #[test]
    fn test_invalid_date_format_contains_letters() {
        // 测试包含字母（应该失败）
        let result = validate_delivery_date("2024-01-abc");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
        }
    }

    #[test]
    fn test_invalid_date_format_short_year() {
        // 测试短年份（应该失败）
        let result = validate_delivery_date("24-01-01");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
        }
    }

    #[test]
    fn test_invalid_date_format_with_spaces() {
        // 测试包含空格（应该失败）
        let result = validate_delivery_date("2024-01-01 ");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
        }
    }

    #[test]
    fn test_invalid_date_format_incomplete() {
        // 测试不完整的日期（应该失败）
        let result = validate_delivery_date("2024-01");
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "invalid_delivery_date_format");
        }
    }

    #[test]
    fn test_valid_date_leap_year() {
        // 测试闰年的2月29日（如果今年是闰年且今天在2月29日之前，应该通过）
        let leap_date = NaiveDate::from_ymd_opt(2024, 2, 29).expect("有效的闰年日期");
        let leap_str = leap_date.format("%Y-%m-%d").to_string();
        let today = Utc::now().date_naive();
        
        if leap_date >= today {
            assert!(validate_delivery_date(&leap_str).is_ok());
        } else {
            let result = validate_delivery_date(&leap_str);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_valid_date_year_boundary() {
        // 测试年份边界（未来年份应该通过）
        let future_year = Utc::now().date_naive() + Duration::days(365);
        let future_str = future_year.format("%Y-%m-%d").to_string();
        assert!(validate_delivery_date(&future_str).is_ok());
    }
}
