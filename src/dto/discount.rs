use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerDiscountResponseDTO {
    #[serde(rename = "discountId")]
    pub discount_id: i64,

    #[serde(rename = "categoryId")]
    pub category_id: i64,

    #[serde(rename = "categoryName")]
    pub category_name: String,

    #[serde(rename = "discountRate")]
    pub discount_rate: Decimal,

    #[serde(rename = "startDate")]
    pub start_date: NaiveDate,

    #[serde(rename = "endDate")]
    pub end_date: NaiveDate,

    pub status: i32,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Validate, Deserialize, FromRow)]
pub struct CustomerDiscountUpdateRequestDTO {
    #[serde(rename = "discountRate")]
    #[validate(custom(function = validate_discount_rate))]
    pub discount_rate: Decimal,

    #[serde(rename = "changedBy")]
    #[validate(length(min = 1, max = 32))]
    pub changed_by: String,
}

#[derive(Debug, Clone, Serialize, Validate, Deserialize, FromRow)]
pub struct CustomerDiscountCreateDTO {
    #[serde(rename = "categoryId")]
    pub category_id: i64,

    #[serde(rename = "discountRate")]
    #[validate(custom(function = validate_discount_rate))]
    pub discount_rate: Decimal,

    #[serde(rename = "startDate")]
    pub start_date: String,

    #[serde(rename = "endDate")]
    pub end_date: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CustomerDiscountCreateRequestDTO {
    #[serde(rename = "createdBy")]
    #[validate(length(min = 1, max = 32))]
    pub created_by: String,

    pub discounts: Vec<CustomerDiscountCreateDTO>,
}

fn validate_discount_rate(rate: &Decimal) -> Result<(), ValidationError> {
    let min = Decimal::new(10, 2); // 0.1
    let max = Decimal::new(100, 2); // 1.0

    if *rate < min || *rate > max {
        return Err(ValidationError::new("discount_rate_range"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;
    use serde_json::json;
    use validator::Validate;

    // Test constants for better maintainability
    const VALID_DISCOUNT_RATE_MIN: &str = "0.10"; // 10%
    const VALID_DISCOUNT_RATE_MAX: &str = "1.00"; // 100%
    const VALID_CATEGORY_ID: i64 = 1;
    const VALID_CHANGED_BY: &str = "admin";
    const VALID_CREATED_BY: &str = "system";

    // Helper functions to reduce code duplication
    fn create_decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    fn create_test_response_dto() -> CustomerDiscountResponseDTO {
        CustomerDiscountResponseDTO {
            discount_id: 1,
            category_id: VALID_CATEGORY_ID,
            category_name: "Electronics".to_string(),
            discount_rate: create_decimal(VALID_DISCOUNT_RATE_MIN),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            status: 1,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        }
    }

    fn create_update_request(rate: &str, changed_by: &str) -> CustomerDiscountUpdateRequestDTO {
        let json_data = json!({
            "discountRate": rate,
            "changedBy": changed_by
        });
        serde_json::from_value(json_data).unwrap()
    }

    fn create_discount_dto(category_id: i64, rate: &str) -> CustomerDiscountCreateDTO {
        let json_data = json!({
            "categoryId": category_id,
            "discountRate": rate,
            "startDate": "2024-01-01",
            "endDate": "2024-12-31"
        });
        serde_json::from_value(json_data).unwrap()
    }

    fn create_create_request(
        created_by: &str,
        discounts: Vec<CustomerDiscountCreateDTO>,
    ) -> CustomerDiscountCreateRequestDTO {
        CustomerDiscountCreateRequestDTO {
            created_by: created_by.to_string(),
            discounts,
        }
    }

    // Tests for validate_discount_rate function
    mod discount_rate_validation_tests {
        use super::*;

        #[test]
        fn test_valid_discount_rates() {
            let valid_rates = vec![
                "0.10", // Minimum valid rate (10%)
                "0.50", // Middle rate (50%)
                "1.00", // Maximum valid rate (100%)
            ];

            for rate_str in valid_rates {
                let rate = create_decimal(rate_str);
                assert!(
                    validate_discount_rate(&rate).is_ok(),
                    "Rate {} should be valid",
                    rate_str
                );
            }
        }

        #[test]
        fn test_invalid_discount_rates() {
            let invalid_rates = vec![
                ("0.09", "Below minimum"),  // 9% - too low
                ("1.01", "Above maximum"),  // 101% - too high
                ("0.00", "Zero rate"),      // 0% - too low
                ("2.00", "Double maximum"), // 200% - way too high
            ];

            for (rate_str, description) in invalid_rates {
                let rate = create_decimal(rate_str);
                assert!(
                    validate_discount_rate(&rate).is_err(),
                    "Rate {} should be invalid: {}",
                    rate_str,
                    description
                );
            }
        }

        #[test]
        fn test_boundary_values() {
            // Test exact boundary values
            let min_rate = Decimal::new(10, 2); // 0.10
            let max_rate = Decimal::new(100, 2); // 1.00

            assert!(validate_discount_rate(&min_rate).is_ok());
            assert!(validate_discount_rate(&max_rate).is_ok());

            // Test just outside boundaries
            let below_min = Decimal::new(9, 2); // 0.09
            let above_max = Decimal::new(101, 2); // 1.01

            assert!(validate_discount_rate(&below_min).is_err());
            assert!(validate_discount_rate(&above_max).is_err());
        }
    }

    // Tests for CustomerDiscountResponseDTO
    mod response_dto_tests {
        use super::*;

        #[test]
        fn test_serialization_field_names() {
            let dto = create_test_response_dto();
            let json_value = serde_json::to_value(&dto).unwrap();

            // Test all renamed fields are present
            assert!(json_value.get("discountId").is_some());
            assert!(json_value.get("categoryId").is_some());
            assert!(json_value.get("categoryName").is_some());
            assert!(json_value.get("discountRate").is_some());
            assert!(json_value.get("startDate").is_some());
            assert!(json_value.get("endDate").is_some());
            assert!(json_value.get("createdAt").is_some());
            assert!(json_value.get("updatedAt").is_some());

            // Test original field names are not present
            assert!(json_value.get("discount_id").is_none());
            assert!(json_value.get("category_id").is_none());
            assert!(json_value.get("created_at").is_none());
            assert!(json_value.get("updated_at").is_none());
        }

        #[test]
        fn test_deserialization() {
            let json_data = json!({
                "discountId": 1,
                "categoryId": 2,
                "categoryName": "Books",
                "discountRate": "0.25",
                "startDate": "2024-01-01",
                "endDate": "2024-12-31",
                "status": 1,
                "createdAt": "2024-01-01T00:00:00Z",
                "updatedAt": "2024-01-01T00:00:00Z"
            });

            let dto: CustomerDiscountResponseDTO = serde_json::from_value(json_data).unwrap();
            assert_eq!(dto.discount_id, 1);
            assert_eq!(dto.category_id, 2);
            assert_eq!(dto.category_name, "Books");
            assert_eq!(dto.discount_rate, create_decimal("0.25"));
        }

        #[test]
        fn test_clone_and_debug() {
            let dto = create_test_response_dto();
            let cloned_dto = dto.clone();

            assert_eq!(dto.discount_id, cloned_dto.discount_id);
            assert_eq!(dto.category_name, cloned_dto.category_name);

            // Test debug output contains key information
            let debug_str = format!("{:?}", dto);
            assert!(debug_str.contains("CustomerDiscountResponseDTO"));
            assert!(debug_str.contains("Electronics"));
        }
    }

    // Tests for CustomerDiscountUpdateRequestDTO
    mod update_request_tests {
        use super::*;

        #[test]
        fn test_valid_update_request() {
            let update_req = create_update_request(VALID_DISCOUNT_RATE_MIN, VALID_CHANGED_BY);
            assert!(update_req.validate().is_ok());
        }

        #[test]
        fn test_invalid_discount_rate() {
            let update_req = create_update_request("0.05", VALID_CHANGED_BY); // Too low
            assert!(update_req.validate().is_err());

            let update_req = create_update_request("1.50", VALID_CHANGED_BY); // Too high
            assert!(update_req.validate().is_err());
        }

        #[test]
        fn test_changed_by_validation() {
            // Test empty string
            let update_req = create_update_request(VALID_DISCOUNT_RATE_MIN, "");
            assert!(update_req.validate().is_err());

            // Test too long string (33 characters)
            let long_name = "a".repeat(33);
            let update_req = create_update_request(VALID_DISCOUNT_RATE_MIN, &long_name);
            assert!(update_req.validate().is_err());

            // Test boundary values
            let min_name = "a"; // 1 character
            let update_req = create_update_request(VALID_DISCOUNT_RATE_MIN, min_name);
            assert!(update_req.validate().is_ok());

            let max_name = "a".repeat(32); // 32 characters
            let update_req = create_update_request(VALID_DISCOUNT_RATE_MIN, &max_name);
            assert!(update_req.validate().is_ok());
        }

        #[test]
        fn test_serialization_field_names() {
            let update_req = CustomerDiscountUpdateRequestDTO {
                discount_rate: create_decimal(VALID_DISCOUNT_RATE_MIN),
                changed_by: VALID_CHANGED_BY.to_string(),
            };

            let json_value = serde_json::to_value(&update_req).unwrap();
            assert!(json_value.get("discountRate").is_some());
            assert!(json_value.get("changedBy").is_some());
            assert!(json_value.get("discount_rate").is_none());
            assert!(json_value.get("changed_by").is_none());
        }
    }

    // Tests for CustomerDiscountCreateDTO
    mod create_dto_tests {
        use super::*;

        #[test]
        fn test_valid_create_dto() {
            let create_dto = create_discount_dto(VALID_CATEGORY_ID, VALID_DISCOUNT_RATE_MIN);
            assert!(create_dto.validate().is_ok());
        }

        #[test]
        fn test_invalid_discount_rate() {
            let create_dto = create_discount_dto(VALID_CATEGORY_ID, "0.05");
            assert!(create_dto.validate().is_err());
        }

        #[test]
        fn test_date_string_handling() {
            let json_data = json!({
                "categoryId": 1,
                "discountRate": "0.25",
                "startDate": "2024-06-01",
                "endDate": "2024-06-30"
            });

            let dto: CustomerDiscountCreateDTO = serde_json::from_value(json_data).unwrap();
            assert_eq!(dto.start_date, "2024-06-01");
            assert_eq!(dto.end_date, "2024-06-30");
        }
    }

    // Tests for CustomerDiscountCreateRequestDTO
    mod create_request_tests {
        use super::*;

        #[test]
        fn test_valid_create_request() {
            let discount = create_discount_dto(VALID_CATEGORY_ID, VALID_DISCOUNT_RATE_MIN);
            let create_req = create_create_request(VALID_CREATED_BY, vec![discount]);
            assert!(create_req.validate().is_ok());
        }

        #[test]
        fn test_created_by_validation() {
            let discount = create_discount_dto(VALID_CATEGORY_ID, VALID_DISCOUNT_RATE_MIN);

            // Test empty created_by
            let create_req = create_create_request("", vec![discount.clone()]);
            assert!(create_req.validate().is_err());

            // Test too long created_by
            let long_name = "a".repeat(33);
            let create_req = create_create_request(&long_name, vec![discount]);
            assert!(create_req.validate().is_err());
        }

        #[test]
        fn test_multiple_discounts() {
            let discount1 = create_discount_dto(1, VALID_DISCOUNT_RATE_MIN);
            let discount2 = create_discount_dto(2, VALID_DISCOUNT_RATE_MAX);

            let create_req = create_create_request(VALID_CREATED_BY, vec![discount1, discount2]);
            assert!(create_req.validate().is_ok());
        }

        #[test]
        fn test_empty_discounts_list() {
            let create_req = create_create_request(VALID_CREATED_BY, vec![]);
            // Should be valid even with empty discounts list
            assert!(create_req.validate().is_ok());
        }

        #[test]
        fn test_nested_validation_failure() {
            // Create a discount with invalid rate
            let invalid_discount = create_discount_dto(VALID_CATEGORY_ID, "0.05"); // Invalid rate
            let create_req = create_create_request(VALID_CREATED_BY, vec![invalid_discount]);

            // The outer validation should pass, but nested validation should fail
            assert!(create_req.validate().is_ok()); // Only validates top-level fields

            // To test nested validation, we need to validate each discount separately
            for discount in &create_req.discounts {
                assert!(discount.validate().is_err());
            }
        }
    }

    // Integration tests
    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_workflow_serialization() {
            // Test the full workflow: create -> update -> response

            // 1. Create request
            let discount = create_discount_dto(1, "0.20");
            let create_req = create_create_request("admin", vec![discount]);

            let create_json = serde_json::to_string(&create_req).unwrap();
            let deserialized_create: CustomerDiscountCreateRequestDTO =
                serde_json::from_str(&create_json).unwrap();

            assert_eq!(create_req.created_by, deserialized_create.created_by);
            assert_eq!(
                create_req.discounts.len(),
                deserialized_create.discounts.len()
            );

            // 2. Update request
            let update_req = create_update_request("0.30", "manager");
            let update_json = serde_json::to_string(&update_req).unwrap();
            let deserialized_update: CustomerDiscountUpdateRequestDTO =
                serde_json::from_str(&update_json).unwrap();

            assert_eq!(update_req.changed_by, deserialized_update.changed_by);

            // 3. Response DTO
            let response = create_test_response_dto();
            let response_json = serde_json::to_string(&response).unwrap();
            let deserialized_response: CustomerDiscountResponseDTO =
                serde_json::from_str(&response_json).unwrap();

            assert_eq!(response.category_name, deserialized_response.category_name);
        }

        #[test]
        fn test_decimal_precision_handling() {
            let test_cases = vec![
                ("0.10", "Basic decimal"),
                ("0.123", "Three decimal places"),
                ("0.9999", "Four decimal places"),
                ("1.00", "Whole number with decimals"),
            ];

            for (rate_str, description) in test_cases {
                let rate = create_decimal(rate_str);
                let update_req = CustomerDiscountUpdateRequestDTO {
                    discount_rate: rate,
                    changed_by: "test".to_string(),
                };

                let json = serde_json::to_string(&update_req).unwrap();
                let deserialized: CustomerDiscountUpdateRequestDTO =
                    serde_json::from_str(&json).unwrap();

                assert_eq!(
                    update_req.discount_rate, deserialized.discount_rate,
                    "Decimal precision should be preserved for: {}",
                    description
                );
            }
        }
    }
}
