use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::types::Decimal;
use sqlx::FromRow;
use validator::Validate;

use crate::utils::validator::validate_apprive_status;

/// 价格公示数据 DTO
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PriceAnnouncement {
    #[serde(rename = "productCode")]
    pub product_code: String,

    #[sqlx(rename = "level1_category")]
    #[serde(rename = "levelOneCategory")]
    pub level_one_category: String,

    #[sqlx(rename = "level3_category")]
    #[serde(rename = "levelThreeCategory")]
    pub level_three_category: String,

    #[serde(rename = "productName")]
    pub product_name: String,

    #[serde(rename = "minPrice")]
    pub min_price: Decimal,

    #[serde(rename = "minPriceChange")]
    pub min_price_change: Decimal,

    #[serde(rename = "avgPrice")]
    pub avg_price: Decimal,

    #[serde(rename = "avgPriceChange")]
    pub avg_price_change: Decimal,

    #[serde(rename = "maxPrice")]
    pub max_price: Decimal,

    #[serde(rename = "maxPriceChange")]
    pub max_price_change: Decimal,

    pub unit: String,

    #[serde(rename = "status")]
    pub price_status: String,

    #[serde(rename = "priceDate")]
    pub price_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Validate)]
pub(crate) struct PriceApprovalParam {
    #[validate(custom(function = validate_apprive_status))]
    pub(crate) status: String,
    pub(crate) products: Vec<i32>,

    #[validate(length(min = 4, max = 32))]
    pub(crate) remark: Option<String>,
}

// 批量创建产品价格 DTO
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct PriceCreateDTO {
    #[serde(rename = "productId")]
    pub(crate) product_id: i32,

    #[serde(rename = "minPrice")]
    pub(crate) min_price: Decimal,

    #[serde(rename = "minPriceDiff")]
    pub(crate) min_price_change: Decimal,

    #[serde(rename = "avgPrice")]
    pub(crate) avg_price: Decimal,

    #[serde(rename = "avgPriceDiff")]
    pub(crate) avg_price_change: Decimal,

    #[serde(rename = "maxPrice")]
    pub(crate) max_price: Decimal,

    #[serde(rename = "maxPriceDiff")]
    pub(crate) max_price_change: Decimal,
}

/// 价格查询参数 DTO
#[derive(Debug, Deserialize, Serialize, Default)]
pub(crate) struct PriceQueryParams {
    /// 价格日期：YYYY-MM-DD，不传默认今天
    pub(crate) date: Option<String>,

    /// 一级分类ID
    #[serde(rename = "category1")]
    pub(crate) category_l1: Option<i32>,

    /// 三级分类ID
    #[serde(rename = "category3")]
    pub(crate) category_l3: Option<i32>,

    /// 产品名称（模糊查询）
    pub(crate) name: Option<String>,
}

impl PriceQueryParams {
    /// 验证查询参数
    pub(crate) fn validate(&self) -> Result<(), String> {
        // 如果提供了日期，验证日期格式
        if let Some(date_str) = &self.date {
            if NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err() {
                return Err("Invalid date format. Use YYYY-MM-DD".to_string());
            }
        }

        // 如果提供了分类ID，验证是否为正数
        if let Some(cat1) = self.category_l1 {
            if cat1 <= 0 {
                return Err("Category1 ID must be positive".to_string());
            }
        }

        if let Some(cat3) = self.category_l3 {
            if cat3 <= 0 {
                return Err("Category3 ID must be positive".to_string());
            }
        }

        // 如果提供了产品名称，验证长度
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                return Err("Product name cannot be empty".to_string());
            }
            if name.len() > 50 {
                return Err("Product name too long".to_string());
            }
        }

        Ok(())
    }
}

/// Pagination parameters for querying prices by status.
///
/// This struct is used in API requests to filter and paginate price data based on status.
/// It supports optional fields with sensible defaults for common use cases, such as fetching
/// pending prices on the first page with a standard page size.
///
/// # Fields
///
/// * `status` - Optional price status filter (e.g., "PENDING", "APPROVED"). Defaults to "PENDING".
/// * `page` - Optional page number for pagination. Starts from 1. Defaults to 1.
/// * `page_size` - Optional number of items per page. Defaults to 10.
///
/// # Example
///
/// ```http
/// GET /api/v1/product_prices?status=PUBLISHED&page=2&pageSize=20
/// ```
///
/// This query fetches published product prices on page 2 with 20 items per page.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PriceStatusPaginationParams {
    pub(crate) status: Option<String>,
    pub(crate) page: Option<i32>,

    #[serde(rename = "pageSize")]
    pub(crate) page_size: Option<i32>,
}

impl Default for PriceStatusPaginationParams {
    fn default() -> Self {
        Self {
            status: Some("PENDING".to_string()),
            page: Some(1),
            page_size: Some(10),
        }
    }
}

impl PriceStatusPaginationParams {
    pub(crate) fn merge_from(&mut self, other: &Self) {
        if other.status.is_some() {
            self.status = other.status.clone();
        }
        if other.page.is_some() {
            self.page = other.page;
        }
        if other.page_size.is_some() {
            self.page_size = other.page_size;
        }
    }
}

// 为了方便测试，实现一个简单的构建器
#[cfg(test)]
impl PriceQueryParams {
    pub fn builder() -> PriceQueryParamsBuilder {
        PriceQueryParamsBuilder::default()
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct PriceQueryParamsBuilder {
    params: PriceQueryParams,
}

#[cfg(test)]
impl PriceQueryParamsBuilder {
    pub fn date(mut self, date: impl Into<String>) -> Self {
        self.params.date = Some(date.into());
        self
    }

    pub fn category1(mut self, id: i32) -> Self {
        self.params.category_l1 = Some(id);
        self
    }

    pub fn category3(mut self, id: i32) -> Self {
        self.params.category_l3 = Some(id);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.params.name = Some(name.into());
        self
    }

    pub fn build(self) -> PriceQueryParams {
        self.params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use serde_json::json;

    // Test constants for better maintainability
    const VALID_DATE: &str = "2024-03-20";
    const INVALID_DATE_FORMAT: &str = "2024/03/20";
    const VALID_CATEGORY_ID: i32 = 1;
    const INVALID_CATEGORY_ID: i32 = -1;
    const VALID_PRODUCT_NAME: &str = "大白菜";
    const EMPTY_PRODUCT_NAME: &str = "";
    const LONG_PRODUCT_NAME: &str =
        "这是一个非常非常长的产品名称用来测试长度限制验证功能是否正常工作超过五十个字符";

    // Helper functions to create test data
    fn create_decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    fn create_test_price_announcement() -> PriceAnnouncement {
        PriceAnnouncement {
            product_code: "P001".to_string(),
            level_one_category: "蔬菜".to_string(),
            level_three_category: "叶菜类".to_string(),
            product_name: "大白菜".to_string(),
            min_price: create_decimal("2.50"),
            min_price_change: create_decimal("0.10"),
            avg_price: create_decimal("3.00"),
            avg_price_change: create_decimal("0.05"),
            max_price: create_decimal("3.50"),
            max_price_change: create_decimal("-0.05"),
            unit: "kg".to_string(),
            price_status: "PUBLISHED".to_string(),
            price_date: Some(NaiveDate::from_ymd_opt(2024, 3, 20).unwrap()),
        }
    }

    fn create_test_aprox_price_param() -> PriceApprovalParam {
        PriceApprovalParam {
            status: "APPROVED".to_string(),
            remark: Some("价格合理".to_string()),
            products: vec![1, 2, 3],
        }
    }

    // Tests for PriceAnnouncement
    mod price_announcement_tests {
        use super::*;

        #[test]
        fn test_price_announcement_serialization() {
            let announcement = create_test_price_announcement();
            let json = serde_json::to_value(&announcement).unwrap();

            // Test field renaming for JSON serialization
            assert!(json.get("productCode").is_some());
            assert!(json.get("levelOneCategory").is_some());
            assert!(json.get("levelThreeCategory").is_some());
            assert!(json.get("productName").is_some());
            assert!(json.get("minPrice").is_some());
            assert!(json.get("minPriceChange").is_some());
            assert!(json.get("avgPrice").is_some());
            assert!(json.get("avgPriceChange").is_some());
            assert!(json.get("maxPrice").is_some());
            assert!(json.get("maxPriceChange").is_some());
            assert!(json.get("status").is_some());
            assert!(json.get("priceDate").is_some());

            // Test original field names are not present
            assert!(json.get("product_code").is_none());
            assert!(json.get("level_one_category").is_none());
            assert!(json.get("level_three_category").is_none());
            assert!(json.get("price_status").is_none());
            assert!(json.get("price_date").is_none());

            // Test values
            assert_eq!(json["productCode"], "P001");
            assert_eq!(json["status"], "PUBLISHED");
            assert_eq!(json["priceDate"], "2024-03-20");
        }

        #[test]
        fn test_price_announcement_deserialization() {
            let json_data = json!({
                "productCode": "P002",
                "levelOneCategory": "水果",
                "levelThreeCategory": "热带水果",
                "productName": "苹果",
                "minPrice": "5.00",
                "minPriceChange": "0.20",
                "avgPrice": "6.00",
                "avgPriceChange": "0.15",
                "maxPrice": "7.00",
                "maxPriceChange": "0.10",
                "unit": "kg",
                "status": "PENDING",
                "priceDate": "2024-03-21"
            });

            let announcement: PriceAnnouncement = serde_json::from_value(json_data).unwrap();
            assert_eq!(announcement.product_code, "P002");
            assert_eq!(announcement.level_one_category, "水果");
            assert_eq!(announcement.level_three_category, "热带水果");
            assert_eq!(announcement.product_name, "苹果");
            assert_eq!(announcement.price_status, "PENDING");
            assert_eq!(
                announcement.price_date,
                Some(NaiveDate::from_ymd_opt(2024, 3, 21).unwrap())
            );
        }

        #[test]
        fn test_price_announcement_with_none_date() {
            let json_data = json!({
                "productCode": "P003",
                "levelOneCategory": "蔬菜",
                "levelThreeCategory": "根茎类",
                "productName": "萝卜",
                "minPrice": "1.50",
                "minPriceChange": "0.00",
                "avgPrice": "2.00",
                "avgPriceChange": "0.00",
                "maxPrice": "2.50",
                "maxPriceChange": "0.00",
                "unit": "kg",
                "status": "DRAFT",
                "priceDate": null
            });

            let announcement: PriceAnnouncement = serde_json::from_value(json_data).unwrap();
            assert_eq!(announcement.product_code, "P003");
            assert_eq!(announcement.price_status, "DRAFT");
            assert!(announcement.price_date.is_none());
        }

        #[test]
        fn test_decimal_precision_handling() {
            let announcement = PriceAnnouncement {
                product_code: "P004".to_string(),
                level_one_category: "测试".to_string(),
                level_three_category: "测试".to_string(),
                product_name: "测试产品".to_string(),
                min_price: create_decimal("12.345"),
                min_price_change: create_decimal("-0.123"),
                avg_price: create_decimal("15.678"),
                avg_price_change: create_decimal("0.456"),
                max_price: create_decimal("18.999"),
                max_price_change: create_decimal("1.234"),
                unit: "kg".to_string(),
                price_status: "PUBLISHED".to_string(),
                price_date: None,
            };

            let json = serde_json::to_string(&announcement).unwrap();
            let deserialized: PriceAnnouncement = serde_json::from_str(&json).unwrap();

            assert_eq!(announcement.min_price, deserialized.min_price);
            assert_eq!(announcement.min_price_change, deserialized.min_price_change);
            assert_eq!(announcement.avg_price, deserialized.avg_price);
            assert_eq!(announcement.avg_price_change, deserialized.avg_price_change);
            assert_eq!(announcement.max_price, deserialized.max_price);
            assert_eq!(announcement.max_price_change, deserialized.max_price_change);
        }
    }

    // Tests for AproxPriceParam
    mod aprox_price_param_tests {
        use super::*;

        #[test]
        fn test_aprox_price_param_serialization() {
            let param = create_test_aprox_price_param();
            let json = serde_json::to_value(&param).unwrap();

            assert_eq!(json["status"], "APPROVED");
            assert_eq!(json["remark"], "价格合理");
            assert_eq!(json["products"].as_array().unwrap().len(), 3);
            assert_eq!(json["products"][0], 1);
            assert_eq!(json["products"][1], 2);
            assert_eq!(json["products"][2], 3);
        }

        #[test]
        fn test_aprox_price_param_with_none_remark() {
            let param = PriceApprovalParam {
                status: "REJECTED".to_string(),
                remark: None,
                products: vec![5, 10],
            };

            let json = serde_json::to_value(&param).unwrap();
            assert_eq!(json["status"], "REJECTED");
            assert!(json["remark"].is_null());
            assert_eq!(json["products"].as_array().unwrap().len(), 2);
        }

        #[test]
        fn test_aprox_price_param_empty_products() {
            let param = PriceApprovalParam {
                status: "PENDING".to_string(),
                remark: Some("无产品".to_string()),
                products: vec![],
            };

            let json = serde_json::to_value(&param).unwrap();
            assert_eq!(json["status"], "PENDING");
            assert_eq!(json["remark"], "无产品");
            assert_eq!(json["products"].as_array().unwrap().len(), 0);
        }

        #[test]
        fn test_aprox_price_param_deserialization() {
            let json_data = json!({
                "status": "PUBLISHED",
                "remark": "批量发布",
                "products": [100, 200, 300, 400]
            });

            let param: PriceApprovalParam = serde_json::from_value(json_data).unwrap();
            assert_eq!(param.status, "PUBLISHED");
            assert_eq!(param.remark, Some("批量发布".to_string()));
            assert_eq!(param.products, vec![100, 200, 300, 400]);
        }
    }

    // Tests for PriceQueryParams validation
    mod price_query_params_validation_tests {
        use super::*;

        #[test]
        fn test_valid_query_params() {
            let params = PriceQueryParams::builder()
                .date(VALID_DATE)
                .category1(VALID_CATEGORY_ID)
                .category3(3)
                .name(VALID_PRODUCT_NAME)
                .build();
            assert!(params.validate().is_ok());
        }

        #[test]
        fn test_all_none_params() {
            let params = PriceQueryParams::default();
            assert!(params.validate().is_ok());
        }

        #[test]
        fn test_only_date_param() {
            let params = PriceQueryParams::builder().date(VALID_DATE).build();
            assert!(params.validate().is_ok());
        }

        #[test]
        fn test_only_category_params() {
            let params = PriceQueryParams::builder()
                .category1(1)
                .category3(3)
                .build();
            assert!(params.validate().is_ok());
        }

        #[test]
        fn test_only_name_param() {
            let params = PriceQueryParams::builder().name(VALID_PRODUCT_NAME).build();
            assert!(params.validate().is_ok());
        }

        #[test]
        fn test_invalid_date_format() {
            let invalid_dates = vec![
                INVALID_DATE_FORMAT, // slash format
                "20240320",          // no separators
                "2024/03/20",        // wrong separators
                "March 20, 2024",    // text format
                "invalid-date",      // completely invalid
                "",                  // empty string
                "2024-13-01",        // invalid month
                "2024-02-30",        // invalid day for February
            ];

            for invalid_date in invalid_dates {
                let params = PriceQueryParams::builder().date(invalid_date).build();
                assert!(
                    params.validate().is_err(),
                    "Date '{}' should be invalid",
                    invalid_date
                );
            }
        }

        #[test]
        fn test_invalid_category_ids() {
            let invalid_ids = vec![INVALID_CATEGORY_ID, 0, -100];

            for invalid_id in invalid_ids {
                // Test category1
                let params = PriceQueryParams::builder().category1(invalid_id).build();
                assert!(
                    params.validate().is_err(),
                    "Category1 ID {} should be invalid",
                    invalid_id
                );

                // Test category3
                let params = PriceQueryParams::builder().category3(invalid_id).build();
                assert!(
                    params.validate().is_err(),
                    "Category3 ID {} should be invalid",
                    invalid_id
                );
            }
        }

        #[test]
        fn test_valid_category_boundary_values() {
            let valid_ids = vec![1, 100, 999999];

            for valid_id in valid_ids {
                let params = PriceQueryParams::builder()
                    .category1(valid_id)
                    .category3(valid_id)
                    .build();
                assert!(
                    params.validate().is_ok(),
                    "Category ID {} should be valid",
                    valid_id
                );
            }
        }

        #[test]
        fn test_invalid_product_names() {
            // Test empty string
            let params = PriceQueryParams::builder().name(EMPTY_PRODUCT_NAME).build();
            assert!(params.validate().is_err());

            // Test whitespace only
            let params = PriceQueryParams::builder().name("   ").build();
            assert!(params.validate().is_err());

            // Test too long name
            let params = PriceQueryParams::builder().name(LONG_PRODUCT_NAME).build();
            assert!(params.validate().is_err());
        }

        #[test]
        fn test_valid_product_name_boundary_values() {
            // Test minimum length (1 character)
            let params = PriceQueryParams::builder().name("a").build();
            assert!(params.validate().is_ok());

            // Test maximum length (50 characters)
            let max_length_name = "a".repeat(50);
            let params = PriceQueryParams::builder().name(&max_length_name).build();
            assert!(params.validate().is_ok());

            // Test with Chinese characters
            let chinese_name = "大白菜小白菜";
            let params = PriceQueryParams::builder().name(chinese_name).build();
            assert!(params.validate().is_ok());
        }

        #[test]
        fn test_validation_error_messages() {
            // Test date format error
            let params = PriceQueryParams::builder()
                .date(INVALID_DATE_FORMAT)
                .build();
            let error = params.validate().unwrap_err();
            assert!(error.contains("Invalid date format"));

            // Test category1 error
            let params = PriceQueryParams::builder()
                .category1(INVALID_CATEGORY_ID)
                .build();
            let error = params.validate().unwrap_err();
            assert!(error.contains("Category1 ID must be positive"));

            // Test category3 error
            let params = PriceQueryParams::builder()
                .category3(INVALID_CATEGORY_ID)
                .build();
            let error = params.validate().unwrap_err();
            assert!(error.contains("Category3 ID must be positive"));

            // Test empty name error
            let params = PriceQueryParams::builder().name(EMPTY_PRODUCT_NAME).build();
            let error = params.validate().unwrap_err();
            assert!(error.contains("Product name cannot be empty"));

            // Test long name error
            let params = PriceQueryParams::builder().name(LONG_PRODUCT_NAME).build();
            let error = params.validate().unwrap_err();
            assert!(error.contains("Product name too long"));
        }

        #[test]
        fn test_multiple_validation_errors() {
            // Only the first error should be returned
            let params = PriceQueryParams {
                date: Some(INVALID_DATE_FORMAT.to_string()),
                category_l1: Some(INVALID_CATEGORY_ID),
                category_l3: Some(INVALID_CATEGORY_ID),
                name: Some(EMPTY_PRODUCT_NAME.to_string()),
            };

            let error = params.validate().unwrap_err();
            // Should return the first error (date format)
            assert!(error.contains("Invalid date format"));
        }
    }

    // Tests for QueryPriceByStatusParams
    mod query_price_by_status_params_tests {
        use super::*;

        #[test]
        fn test_default_values() {
            let params = PriceStatusPaginationParams::default();
            assert_eq!(params.status, Some("PENDING".to_string()));
            assert_eq!(params.page, Some(1));
            assert_eq!(params.page_size, Some(10));
        }

        #[test]
        fn test_serialization_field_mapping() {
            let params = PriceStatusPaginationParams {
                status: Some("APPROVED".to_string()),
                page: Some(2),
                page_size: Some(20),
            };

            let json = serde_json::to_value(&params).unwrap();
            assert_eq!(json["status"], "APPROVED");
            assert_eq!(json["page"], 2);
            assert_eq!(json["pageSize"], 20);

            // Test original field name is not present
            assert!(json.get("page_size").is_none());
        }

        #[test]
        fn test_deserialization() {
            let json_data = json!({
                "status": "PUBLISHED",
                "page": 3,
                "pageSize": 50
            });

            let params: PriceStatusPaginationParams = serde_json::from_value(json_data).unwrap();
            assert_eq!(params.status, Some("PUBLISHED".to_string()));
            assert_eq!(params.page, Some(3));
            assert_eq!(params.page_size, Some(50));
        }

        #[test]
        fn test_optional_fields() {
            let json_data = json!({});
            let params: PriceStatusPaginationParams = serde_json::from_value(json_data).unwrap();
            assert!(params.status.is_none());
            assert!(params.page.is_none());
            assert!(params.page_size.is_none());

            // Test partial fields
            let json_data = json!({
                "status": "REJECTED"
            });
            let params: PriceStatusPaginationParams = serde_json::from_value(json_data).unwrap();
            assert_eq!(params.status, Some("REJECTED".to_string()));
            assert!(params.page.is_none());
            assert!(params.page_size.is_none());
        }

        // TODO: Add validation tests once validation is implemented
        #[test]
        #[ignore = "Validation not yet implemented - see TODO in code"]
        fn test_status_validation_placeholder() {
            // This test should be implemented when status validation is added
            // Expected validations:
            // - Valid statuses: PENDING, APPROVED, REJECTED, PUBLISHED
            // - Invalid statuses: empty string, unknown values
            // - Page validation: positive numbers only
            // - Page size validation: reasonable limits (e.g., 1-1000)
        }
    }

    // Tests for PriceQueryParamsBuilder
    mod builder_tests {
        use super::*;

        #[test]
        fn test_builder_pattern() {
            let params = PriceQueryParams::builder()
                .date("2024-12-31")
                .category1(5)
                .category3(15)
                .name("测试产品")
                .build();

            assert_eq!(params.date, Some("2024-12-31".to_string()));
            assert_eq!(params.category_l1, Some(5));
            assert_eq!(params.category_l3, Some(15));
            assert_eq!(params.name, Some("测试产品".to_string()));
        }

        #[test]
        fn test_builder_partial_build() {
            let params = PriceQueryParams::builder()
                .date("2024-06-15")
                .name("部分参数")
                .build();

            assert_eq!(params.date, Some("2024-06-15".to_string()));
            assert!(params.category_l1.is_none());
            assert!(params.category_l3.is_none());
            assert_eq!(params.name, Some("部分参数".to_string()));
        }

        #[test]
        fn test_builder_chainable() {
            let builder = PriceQueryParams::builder();
            let params = builder
                .date("2024-01-01")
                .category1(1)
                .category3(2)
                .name("链式调用")
                .build();

            assert!(params.validate().is_ok());
        }

        #[test]
        fn test_builder_with_string_types() {
            // Test that builder accepts different string types
            let string_date = String::from("2024-05-01");
            let str_name = "字符串测试";

            let params = PriceQueryParams::builder()
                .date(string_date)
                .name(str_name)
                .build();

            assert_eq!(params.date, Some("2024-05-01".to_string()));
            assert_eq!(params.name, Some("字符串测试".to_string()));
        }
    }

    // Integration tests
    mod integration_tests {
        use super::*;

        #[test]
        fn test_complete_workflow_serialization() {
            // Test a complete workflow from builder to validation to serialization
            let params = PriceQueryParams::builder()
                .date("2024-07-20")
                .category1(10)
                .category3(25)
                .name("完整流程测试")
                .build();

            // Validate
            assert!(params.validate().is_ok());

            // Serialize
            let json = serde_json::to_value(&params).unwrap();
            assert_eq!(json["date"], "2024-07-20");
            assert_eq!(json["category1"], 10);
            assert_eq!(json["category3"], 25);
            assert_eq!(json["name"], "完整流程测试");

            // Deserialize back
            let deserialized: PriceQueryParams = serde_json::from_value(json).unwrap();
            assert_eq!(params.date, deserialized.date);
            assert_eq!(params.category_l1, deserialized.category_l1);
            assert_eq!(params.category_l3, deserialized.category_l3);
            assert_eq!(params.name, deserialized.name);

            // Validate again
            assert!(deserialized.validate().is_ok());
        }

        #[test]
        fn test_real_world_scenarios() {
            // Scenario 1: Search by category only
            let params = PriceQueryParams::builder().category1(1).build();
            assert!(params.validate().is_ok());

            // Scenario 2: Search by name only
            let params = PriceQueryParams::builder().name("白菜").build();
            assert!(params.validate().is_ok());

            // Scenario 3: Search by date only
            let params = PriceQueryParams::builder().date("2024-08-15").build();
            assert!(params.validate().is_ok());

            // Scenario 4: Search with all filters
            let params = PriceQueryParams::builder()
                .date("2024-09-01")
                .category1(2)
                .category3(8)
                .name("胡萝卜")
                .build();
            assert!(params.validate().is_ok());
        }

        #[test]
        fn test_edge_cases_and_boundary_values() {
            // Boundary date values
            let params = PriceQueryParams::builder()
                .date("2024-01-01") // Start of year
                .build();
            assert!(params.validate().is_ok());

            let params = PriceQueryParams::builder()
                .date("2024-12-31") // End of year
                .build();
            assert!(params.validate().is_ok());

            // Boundary category values
            let params = PriceQueryParams::builder()
                .category1(1) // Minimum valid category
                .category3(999999) // Large category ID
                .build();
            assert!(params.validate().is_ok());

            // Boundary name values
            let params = PriceQueryParams::builder()
                .name("a") // Single character
                .build();
            assert!(params.validate().is_ok());

            let params = PriceQueryParams::builder()
                .name(&"x".repeat(50)) // Maximum length
                .build();
            assert!(params.validate().is_ok());
        }
    }
}
