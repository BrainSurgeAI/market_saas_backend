use chrono::NaiveDate;

use serde::{Deserialize, Serialize};
use sqlx::types::Decimal;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProductPriceDTO {
    pub product_code: String,

    #[sqlx(rename = "level1_category")]
    pub level_one_category: String,

    #[sqlx(rename = "level3_category")]
    pub level_three_category: String,
    pub product_name: String,

    pub min_price: Decimal,

    pub min_price_change: Decimal,

    pub avg_price: Decimal,

    pub avg_price_change: Decimal,

    pub max_price: Decimal,

    pub max_price_change: Decimal,

    pub unit: String,

    #[serde(rename = "status")]
    pub price_status: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProductDailyPriceComparisonDTO {
    #[sqlx(rename = "product_id")]
    #[serde(rename = "id")]
    pub product_id: i32,

    #[sqlx(rename = "product_name")]
    #[serde(rename = "name")]
    pub product_name: String,

    pub unit: String,

    #[sqlx(rename = "today_min_price")]
    #[serde(rename = "minPrice")]
    pub min_price: Decimal,

    #[sqlx(rename = "today_avg_price")]
    #[serde(rename = "avgPrice")]
    pub avg_price: Decimal,

    #[sqlx(rename = "today_max_price")]
    #[serde(rename = "maxPrice")]
    pub max_price: Decimal,

    #[sqlx(rename = "assigned_category_name")]
    #[serde(rename = "category")]
    pub category_name: String,

    #[sqlx(rename = "yesterday_min_price")]
    #[serde(rename = "lastMinPrice")]
    pub yesterday_min_price: Decimal,

    #[sqlx(rename = "yesterday_avg_price")]
    #[serde(rename = "lastAvgPrice")]
    pub yesterday_avg_price: Decimal,

    #[sqlx(rename = "yesterday_max_price")]
    #[serde(rename = "lastMaxPrice")]
    pub yesterday_max_price: Decimal,

    #[sqlx(rename = "price_status")]
    #[serde(rename = "status")]
    pub price_status: Option<String>,

    #[serde(rename = "priceSource")]
    pub price_source: Option<String>,

    #[serde(rename = "publishDate")]
    #[serde(skip_deserializing)]
    pub publish_date: Option<NaiveDate>,
}

// 产品详情
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProductDetailDTO {
    #[serde(rename = "id")]
    #[serde(skip_deserializing)]
    pub product_code: String,

    pub name: String,

    pub unit: String,

    #[serde(rename = "description")]
    pub product_description: Option<String>,

    pub brand: Option<String>,

    #[serde(rename = "storageConditions")]
    pub storage_conditions: Option<String>,

    #[serde(rename = "shelfLife")]
    pub shelf_life: Option<String>,

    #[serde(rename = "pricingMethod")]
    pub pricing_method: Option<String>,

    #[serde(rename = "minOrderQuantity")]
    pub min_order_quantity: Option<Decimal>,

    #[serde(rename = "specialNotes")]
    pub special_notes: Option<String>,

    pub tips: Option<String>,

    #[serde(rename = "taxRate")]
    pub tax_rate: Option<Decimal>,

    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProductRequestDTO {
    pub name: Option<String>,

    pub unit: Option<String>,

    #[serde(rename = "description")]
    pub product_description: Option<String>,

    pub brand: Option<String>,

    #[serde(rename = "storageConditions")]
    pub storage_conditions: Option<String>,

    #[serde(rename = "shelfLife")]
    pub shelf_life: Option<String>,

    #[serde(rename = "pricingMethod")]
    pub pricing_method: Option<String>,

    #[serde(rename = "specialNotes")]
    pub special_notes: Option<String>,

    pub tips: Option<String>,

    pub spec: Option<String>,

    #[serde(rename = "taxRate")]
    pub tax_rate: Option<Decimal>,

    #[serde(rename = "minOrderQuantity")]
    pub min_order_quantity: Option<Decimal>,

    #[serde(rename = "isDisabled")]
    pub is_disabled: Option<bool>,

    #[serde(rename = "processingServices")]
    pub processing_services: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductDetailResponse {
    pub product: ProductDetailDTO,

    #[serde(rename = "processingFees")]
    pub processing_fees: Option<Vec<ProcessingFeeDTO>>,
}

// 批量创建产品价格 DTO
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProductPriceCreateDTO {
    #[serde(rename = "productId")]
    pub product_id: i32,

    #[serde(rename = "minPrice")]
    pub min_price: Decimal,

    #[serde(rename = "minPriceDiff")]
    pub min_price_change: Decimal,

    #[serde(rename = "avgPrice")]
    pub avg_price: Decimal,

    #[serde(rename = "avgPriceDiff")]
    pub avg_price_change: Decimal,

    #[serde(rename = "maxPrice")]
    pub max_price: Decimal,

    #[serde(rename = "maxPriceDiff")]
    pub max_price_change: Decimal,
}

// 价格状态统计 DTO
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PriceStatusDTO {
    pub category_id: i32,
    pub category_name: String,
    pub total_products: i32,
    pub products_without_price: i32,
    pub products_pending: i32,
    pub products_approved: i32,
    pub products_published: i32,
    pub products_rejected: i32,
}

/// 价格查询参数 DTO
#[derive(Debug, Deserialize, Default)]
pub struct PriceQueryParams {
    /// 价格日期：YYYY-MM-DD，不传默认今天
    pub date: Option<String>,

    /// 一级分类ID
    #[serde(rename = "category1")]
    pub category_l1: Option<i32>,

    /// 三级分类ID
    #[serde(rename = "category3")]
    pub category_l3: Option<i32>,

    /// 产品名称（模糊查询）
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProcessingFeeDTO {
    #[sqlx(rename = "processing_fee_id")]
    #[serde(rename = "processingFeeId")]
    pub id: i32,

    #[serde(rename = "processingType")]
    pub processing_type: String,

    #[serde(rename = "feeType")]
    pub fee_type: String,

    #[serde(rename = "feeValue")]
    pub fee_value: Decimal,

    pub description: Option<String>,

    #[serde(rename = "isCheckbox")]
    pub is_checkbox: bool,

    #[serde(rename = "isDefault")]
    #[sqlx(default)]
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ProductListQueryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub category_id: Option<i32>,
    pub name: Option<String>,
    pub is_disabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProductListDTO {
    #[serde(rename = "id")]
    pub product_code: String,

    pub name: String,

    #[serde(rename = "categoryId")]
    pub category_level1_id: i32,

    pub category: String,

    #[serde(rename = "price")]
    pub avg_price: Decimal,

    #[serde(rename = "minPrice")]
    pub min_price: Decimal,

    #[serde(rename = "maxPrice")]
    pub max_price: Decimal,

    pub unit: String,
    pub image: Option<String>,

    #[serde(rename = "isDisabled")]
    pub is_disabled: bool,

    #[serde(rename = "discountRate")]
    pub discount_rate: Decimal,

    #[serde(rename = "minOrderQuantity")]
    pub min_order_quantity: Decimal,
}

/// 产品概览 DTO 产品管理页面使用
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProductOverviewDTO {
    #[serde(rename = "id")]
    pub product_code: String,
    pub name: String,
    pub unit: String,
    pub description: Option<String>,
    #[serde(rename = "isDisabled")]
    pub is_disabled: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProductOverviewResponse {
    pub products: Vec<ProductOverviewDTO>,
    pub total: i32,
}

impl PriceQueryParams {
    /// 验证查询参数
    pub fn validate(&self) -> Result<(), String> {
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
    use serde_json::json;
    use rust_decimal::Decimal;
    use chrono::NaiveDate;

    // Test constants for better maintainability
    const VALID_PRODUCT_CODE: &str = "P001";
    const VALID_PRODUCT_NAME: &str = "苹果";
    const VALID_CATEGORY_ID: i32 = 1;
    const VALID_UNIT: &str = "kg";
    const VALID_DATE: &str = "2024-03-20";
    const INVALID_DATE_FORMAT: &str = "2024/03/20";
    const INVALID_CATEGORY_ID: i32 = -1;
    const EMPTY_PRODUCT_NAME: &str = "";
    const LONG_PRODUCT_NAME: &str = "这是一个非常非常长的产品名称用来测试长度限制验证功能是否正常工作超过五十个字符";

    // Helper functions to create test data
    fn create_decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    fn create_test_product_price_dto() -> ProductPriceDTO {
        ProductPriceDTO {
            product_code: VALID_PRODUCT_CODE.to_string(),
            level_one_category: "水果".to_string(),
            level_three_category: "苹果类".to_string(),
            product_name: VALID_PRODUCT_NAME.to_string(),
            min_price: create_decimal("5.00"),
            min_price_change: create_decimal("0.10"),
            avg_price: create_decimal("6.00"),
            avg_price_change: create_decimal("0.05"),
            max_price: create_decimal("7.00"),
            max_price_change: create_decimal("-0.05"),
            unit: VALID_UNIT.to_string(),
            price_status: "PUBLISHED".to_string(),
        }
    }

    fn create_test_product_daily_price_comparison() -> ProductDailyPriceComparisonDTO {
        ProductDailyPriceComparisonDTO {
            product_id: 1,
            product_name: VALID_PRODUCT_NAME.to_string(),
            unit: VALID_UNIT.to_string(),
            min_price: create_decimal("5.00"),
            avg_price: create_decimal("6.00"),
            max_price: create_decimal("7.00"),
            category_name: "水果".to_string(),
            yesterday_min_price: create_decimal("4.80"),
            yesterday_avg_price: create_decimal("5.80"),
            yesterday_max_price: create_decimal("6.80"),
            price_status: Some("PUBLISHED".to_string()),
            price_source: Some("市场价格".to_string()),
            publish_date: Some(NaiveDate::from_ymd_opt(2024, 3, 20).unwrap()),
        }
    }

    fn create_test_product_detail_dto() -> ProductDetailDTO {
        ProductDetailDTO {
            product_code: VALID_PRODUCT_CODE.to_string(),
            name: VALID_PRODUCT_NAME.to_string(),
            unit: VALID_UNIT.to_string(),
            product_description: Some("新鲜苹果".to_string()),
            brand: Some("优质品牌".to_string()),
            storage_conditions: Some("阴凉干燥处".to_string()),
            shelf_life: Some("7天".to_string()),
            pricing_method: Some("按重量".to_string()),
            min_order_quantity: Some(create_decimal("1.0")),
            special_notes: Some("请注意保鲜".to_string()),
            tips: Some("建议冷藏".to_string()),
            tax_rate: Some(create_decimal("0.13")),
            image: Some("apple.jpg".to_string()),
        }
    }

    fn create_test_processing_fee_dto() -> ProcessingFeeDTO {
        ProcessingFeeDTO {
            id: 1,
            processing_type: "切片".to_string(),
            fee_type: "FIXED".to_string(),
            fee_value: create_decimal("2.00"),
            description: Some("切片加工费".to_string()),
            is_checkbox: true,
            is_default: Some(false),
        }
    }

    // Tests for ProductPriceDTO
    mod product_price_dto_tests {
        use super::*;

        #[test]
        fn test_product_price_dto_serialization() {
            let dto = create_test_product_price_dto();
            let json = serde_json::to_value(&dto).unwrap();

            // Test field renaming
            assert!(json.get("status").is_some());
            assert!(json.get("price_status").is_none());

            // Test values
            assert_eq!(json["product_code"], VALID_PRODUCT_CODE);
            assert_eq!(json["level_one_category"], "水果");
            assert_eq!(json["level_three_category"], "苹果类");
            assert_eq!(json["product_name"], VALID_PRODUCT_NAME);
            assert_eq!(json["status"], "PUBLISHED");
        }

        #[test]
        fn test_product_price_dto_deserialization() {
            let json_data = json!({
                "product_code": "P002",
                "level_one_category": "蔬菜",
                "level_three_category": "叶菜类",
                "product_name": "大白菜",
                "min_price": "2.50",
                "min_price_change": "0.20",
                "avg_price": "3.00",
                "avg_price_change": "0.15",
                "max_price": "3.50",
                "max_price_change": "0.10",
                "unit": "kg",
                "status": "PENDING"
            });

            let dto: ProductPriceDTO = serde_json::from_value(json_data).unwrap();
            assert_eq!(dto.product_code, "P002");
            assert_eq!(dto.level_one_category, "蔬菜");
            assert_eq!(dto.level_three_category, "叶菜类");
            assert_eq!(dto.product_name, "大白菜");
            assert_eq!(dto.price_status, "PENDING");
        }

        #[test]
        fn test_decimal_precision_handling() {
            let dto = ProductPriceDTO {
                product_code: "P003".to_string(),
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
            };

            let json = serde_json::to_string(&dto).unwrap();
            let deserialized: ProductPriceDTO = serde_json::from_str(&json).unwrap();

            assert_eq!(dto.min_price, deserialized.min_price);
            assert_eq!(dto.min_price_change, deserialized.min_price_change);
            assert_eq!(dto.avg_price, deserialized.avg_price);
            assert_eq!(dto.avg_price_change, deserialized.avg_price_change);
            assert_eq!(dto.max_price, deserialized.max_price);
            assert_eq!(dto.max_price_change, deserialized.max_price_change);
        }
    }

    // Tests for ProductDailyPriceComparisonDTO
    mod product_daily_price_comparison_tests {
        use super::*;

        #[test]
        fn test_complex_field_mapping() {
            let dto = create_test_product_daily_price_comparison();
            let json = serde_json::to_value(&dto).unwrap();

            // Test sqlx and serde field renaming
            assert!(json.get("id").is_some());
            assert!(json.get("name").is_some());
            assert!(json.get("minPrice").is_some());
            assert!(json.get("avgPrice").is_some());
            assert!(json.get("maxPrice").is_some());
            assert!(json.get("category").is_some());
            assert!(json.get("lastMinPrice").is_some());
            assert!(json.get("lastAvgPrice").is_some());
            assert!(json.get("lastMaxPrice").is_some());
            assert!(json.get("status").is_some());
            assert!(json.get("priceSource").is_some());
            assert!(json.get("publishDate").is_some());

            // Test original field names are not present
            assert!(json.get("product_id").is_none());
            assert!(json.get("product_name").is_none());
            assert!(json.get("today_min_price").is_none());
            assert!(json.get("yesterday_min_price").is_none());
            assert!(json.get("price_status").is_none());

            // Test values
            assert_eq!(json["id"], 1);
            assert_eq!(json["name"], VALID_PRODUCT_NAME);
            assert_eq!(json["minPrice"], "5.00");
            assert_eq!(json["category"], "水果");
            assert_eq!(json["status"], "PUBLISHED");
            assert_eq!(json["publishDate"], "2024-03-20");
        }

        #[test]
        fn test_optional_fields() {
            let dto = ProductDailyPriceComparisonDTO {
                product_id: 2,
                product_name: "测试产品".to_string(),
                unit: "pcs".to_string(),
                min_price: create_decimal("1.00"),
                avg_price: create_decimal("2.00"),
                max_price: create_decimal("3.00"),
                category_name: "测试类别".to_string(),
                yesterday_min_price: create_decimal("0.90"),
                yesterday_avg_price: create_decimal("1.90"),
                yesterday_max_price: create_decimal("2.90"),
                price_status: None,
                price_source: None,
                publish_date: None,
            };

            let json = serde_json::to_value(&dto).unwrap();
            assert!(json["status"].is_null());
            assert!(json["priceSource"].is_null());
            assert!(json["publishDate"].is_null());
        }

        #[test]
        fn test_skip_deserializing_publish_date() {
            // publishDate should be skipped during deserialization
            let json_data = json!({
                "id": 1,
                "name": "测试产品",
                "unit": "kg",
                "minPrice": "5.00",
                "avgPrice": "6.00",
                "maxPrice": "7.00",
                "category": "水果",
                "lastMinPrice": "4.80",
                "lastAvgPrice": "5.80",
                "lastMaxPrice": "6.80",
                "status": "PUBLISHED",
                "priceSource": "市场价格",
                "publishDate": "2024-03-20"  // This should be ignored
            });

            let dto: ProductDailyPriceComparisonDTO = serde_json::from_value(json_data).unwrap();
            assert_eq!(dto.product_id, 1);
            assert_eq!(dto.product_name, "测试产品");
            // publish_date should remain None due to skip_deserializing
            assert!(dto.publish_date.is_none());
        }
    }

    // Tests for ProductDetailDTO
    mod product_detail_dto_tests {
        use super::*;

        #[test]
        fn test_product_detail_serialization() {
            let dto = create_test_product_detail_dto();
            let json = serde_json::to_value(&dto).unwrap();

            // Test field renaming
            assert!(json.get("id").is_some());
            assert!(json.get("description").is_some());
            assert!(json.get("storageConditions").is_some());
            assert!(json.get("shelfLife").is_some());
            assert!(json.get("pricingMethod").is_some());
            assert!(json.get("minOrderQuantity").is_some());
            assert!(json.get("specialNotes").is_some());
            assert!(json.get("taxRate").is_some());

            // Test original field names are not present
            assert!(json.get("product_code").is_none());
            assert!(json.get("product_description").is_none());
            assert!(json.get("storage_conditions").is_none());
            assert!(json.get("shelf_life").is_none());
            assert!(json.get("pricing_method").is_none());
            assert!(json.get("min_order_quantity").is_none());
            assert!(json.get("special_notes").is_none());
            assert!(json.get("tax_rate").is_none());

            assert_eq!(json["id"], VALID_PRODUCT_CODE);
            assert_eq!(json["name"], VALID_PRODUCT_NAME);
            assert_eq!(json["description"], "新鲜苹果");
        }

        #[test]
        fn test_skip_deserializing_product_code() {
            let json_data = json!({
                "id": "P999",  // This should be ignored
                "name": "测试产品",
                "unit": "kg",
                "description": "测试描述",
                "brand": "测试品牌",
                "storageConditions": "常温",
                "shelfLife": "30天",
                "pricingMethod": "按重量",
                "minOrderQuantity": "1.0",
                "specialNotes": "特别注意",
                "tips": "小贴士",
                "taxRate": "0.13",
                "image": "test.jpg"
            });

            let dto: ProductDetailDTO = serde_json::from_value(json_data).unwrap();
            assert_eq!(dto.name, "测试产品");
            assert_eq!(dto.unit, "kg");
            // product_code should use Default value since it's skipped during deserialization
            assert_eq!(dto.product_code, "");
        }

        #[test]
        fn test_optional_fields_handling() {
            let dto = ProductDetailDTO {
                product_code: VALID_PRODUCT_CODE.to_string(),
                name: VALID_PRODUCT_NAME.to_string(),
                unit: VALID_UNIT.to_string(),
                product_description: None,
                brand: None,
                storage_conditions: None,
                shelf_life: None,
                pricing_method: None,
                min_order_quantity: None,
                special_notes: None,
                tips: None,
                tax_rate: None,
                image: None,
            };

            let json = serde_json::to_value(&dto).unwrap();
            assert!(json["description"].is_null());
            assert!(json["brand"].is_null());
            assert!(json["storageConditions"].is_null());
            assert!(json["shelfLife"].is_null());
            assert!(json["pricingMethod"].is_null());
            assert!(json["minOrderQuantity"].is_null());
            assert!(json["specialNotes"].is_null());
            assert!(json["tips"].is_null());
            assert!(json["taxRate"].is_null());
            assert!(json["image"].is_null());
        }
    }

    // Tests for UpdateProductRequestDTO
    mod update_product_request_tests {
        use super::*;

        #[test]
        fn test_all_optional_fields() {
            let dto = UpdateProductRequestDTO {
                name: Some("新产品名".to_string()),
                unit: Some("pcs".to_string()),
                product_description: Some("新描述".to_string()),
                brand: Some("新品牌".to_string()),
                storage_conditions: Some("冷藏".to_string()),
                shelf_life: Some("15天".to_string()),
                pricing_method: Some("固定价格".to_string()),
                special_notes: Some("特殊说明".to_string()),
                tips: Some("使用提示".to_string()),
                spec: Some("规格".to_string()),
                tax_rate: Some(create_decimal("0.13")),
                min_order_quantity: Some(create_decimal("5.0")),
                is_disabled: Some(false),
                processing_services: Some(vec![1, 2, 3]),
            };

            let json = serde_json::to_value(&dto).unwrap();

            // Test field renaming
            assert!(json.get("description").is_some());
            assert!(json.get("storageConditions").is_some());
            assert!(json.get("shelfLife").is_some());
            assert!(json.get("pricingMethod").is_some());
            assert!(json.get("specialNotes").is_some());
            assert!(json.get("taxRate").is_some());
            assert!(json.get("minOrderQuantity").is_some());
            assert!(json.get("isDisabled").is_some());
            assert!(json.get("processingServices").is_some());

            assert_eq!(json["name"], "新产品名");
            assert_eq!(json["isDisabled"], false);
            assert_eq!(json["processingServices"].as_array().unwrap().len(), 3);
        }

        #[test]
        fn test_empty_update_request() {
            let dto = UpdateProductRequestDTO {
                name: None,
                unit: None,
                product_description: None,
                brand: None,
                storage_conditions: None,
                shelf_life: None,
                pricing_method: None,
                special_notes: None,
                tips: None,
                spec: None,
                tax_rate: None,
                min_order_quantity: None,
                is_disabled: None,
                processing_services: None,
            };

            let json = serde_json::to_value(&dto).unwrap();
            // All fields should be null
            assert!(json["name"].is_null());
            assert!(json["unit"].is_null());
            assert!(json["description"].is_null());
            assert!(json["brand"].is_null());
            assert!(json["isDisabled"].is_null());
            assert!(json["processingServices"].is_null());
        }

        #[test]
        fn test_deserialization() {
            let json_data = json!({
                "name": "更新产品",
                "unit": "box",
                "description": "更新描述",
                "isDisabled": true,
                "processingServices": [4, 5, 6]
            });

            let dto: UpdateProductRequestDTO = serde_json::from_value(json_data).unwrap();
            assert_eq!(dto.name, Some("更新产品".to_string()));
            assert_eq!(dto.unit, Some("box".to_string()));
            assert_eq!(dto.product_description, Some("更新描述".to_string()));
            assert_eq!(dto.is_disabled, Some(true));
            assert_eq!(dto.processing_services, Some(vec![4, 5, 6]));
        }
    }

    // Tests for ProductDetailResponse
    mod product_detail_response_tests {
        use super::*;

        #[test]
        fn test_nested_structure() {
            let product = create_test_product_detail_dto();
            let processing_fees = vec![create_test_processing_fee_dto()];
            
            let response = ProductDetailResponse {
                product,
                processing_fees: Some(processing_fees),
            };

            let json = serde_json::to_value(&response).unwrap();
            assert!(json.get("product").is_some());
            assert!(json.get("processingFees").is_some());
            assert_eq!(json["processingFees"].as_array().unwrap().len(), 1);
        }

        #[test]
        fn test_none_processing_fees() {
            let product = create_test_product_detail_dto();
            
            let response = ProductDetailResponse {
                product,
                processing_fees: None,
            };

            let json = serde_json::to_value(&response).unwrap();
            assert!(json.get("product").is_some());
            assert!(json["processingFees"].is_null());
        }
    }

    // Tests for ProductPriceCreateDTO
    mod product_price_create_tests {
        use super::*;

        #[test]
        fn test_field_mapping() {
            let dto = ProductPriceCreateDTO {
                product_id: 1,
                min_price: create_decimal("5.00"),
                min_price_change: create_decimal("0.10"),
                avg_price: create_decimal("6.00"),
                avg_price_change: create_decimal("0.05"),
                max_price: create_decimal("7.00"),
                max_price_change: create_decimal("-0.05"),
            };

            let json = serde_json::to_value(&dto).unwrap();

            // Test field renaming
            assert!(json.get("productId").is_some());
            assert!(json.get("minPrice").is_some());
            assert!(json.get("minPriceDiff").is_some());
            assert!(json.get("avgPrice").is_some());
            assert!(json.get("avgPriceDiff").is_some());
            assert!(json.get("maxPrice").is_some());
            assert!(json.get("maxPriceDiff").is_some());

            // Test original field names are not present
            assert!(json.get("product_id").is_none());
            assert!(json.get("min_price").is_none());
            assert!(json.get("min_price_change").is_none());
            assert!(json.get("avg_price_change").is_none());
            assert!(json.get("max_price_change").is_none());

            assert_eq!(json["productId"], 1);
            assert_eq!(json["minPrice"], "5.00");
            assert_eq!(json["avgPrice"], "6.00");
        }
    }

    // Tests for PriceStatusDTO
    mod price_status_dto_tests {
        use super::*;

        #[test]
        fn test_statistics_data() {
            let dto = PriceStatusDTO {
                category_id: 1,
                category_name: "水果".to_string(),
                total_products: 100,
                products_without_price: 10,
                products_pending: 20,
                products_approved: 30,
                products_published: 35,
                products_rejected: 5,
            };

            let json = serde_json::to_value(&dto).unwrap();
            assert_eq!(json["category_id"], 1);
            assert_eq!(json["category_name"], "水果");
            assert_eq!(json["total_products"], 100);
            assert_eq!(json["products_without_price"], 10);
            assert_eq!(json["products_pending"], 20);
            assert_eq!(json["products_approved"], 30);
            assert_eq!(json["products_published"], 35);
            assert_eq!(json["products_rejected"], 5);

            // Validate totals make sense
            let counted_total = 10 + 20 + 30 + 35 + 5;
            assert_eq!(counted_total, 100);
        }

        #[test]
        fn test_zero_statistics() {
            let dto = PriceStatusDTO {
                category_id: 2,
                category_name: "空类别".to_string(),
                total_products: 0,
                products_without_price: 0,
                products_pending: 0,
                products_approved: 0,
                products_published: 0,
                products_rejected: 0,
            };

            let json = serde_json::to_value(&dto).unwrap();
            assert_eq!(json["total_products"], 0);
            assert_eq!(json["products_without_price"], 0);
        }
    }

    // Tests for ProcessingFeeDTO
    mod processing_fee_dto_tests {
        use super::*;

        #[test]
        fn test_complex_field_mapping() {
            let dto = create_test_processing_fee_dto();
            let json = serde_json::to_value(&dto).unwrap();

            // Test field renaming
            assert!(json.get("processingFeeId").is_some());
            assert!(json.get("processingType").is_some());
            assert!(json.get("feeType").is_some());
            assert!(json.get("feeValue").is_some());
            assert!(json.get("isCheckbox").is_some());
            assert!(json.get("isDefault").is_some());

            // Test original field names are not present
            assert!(json.get("id").is_none());
            assert!(json.get("processing_type").is_none());
            assert!(json.get("fee_type").is_none());
            assert!(json.get("fee_value").is_none());
            assert!(json.get("is_checkbox").is_none());
            assert!(json.get("is_default").is_none());

            assert_eq!(json["processingFeeId"], 1);
            assert_eq!(json["processingType"], "切片");
            assert_eq!(json["feeType"], "FIXED");
            assert_eq!(json["feeValue"], "2.00");
            assert_eq!(json["isCheckbox"], true);
            assert_eq!(json["isDefault"], false);
        }

        #[test]
        fn test_optional_fields() {
            let dto = ProcessingFeeDTO {
                id: 2,
                processing_type: "包装".to_string(),
                fee_type: "PERCENTAGE".to_string(),
                fee_value: create_decimal("0.05"),
                description: None,
                is_checkbox: false,
                is_default: None,
            };

            let json = serde_json::to_value(&dto).unwrap();
            assert!(json["description"].is_null());
            assert!(json["isDefault"].is_null());
            assert_eq!(json["isCheckbox"], false);
        }
    }

    // Tests for ProductListQueryParams
    mod product_list_query_params_tests {
        use super::*;

        #[test]
        fn test_query_params_deserialization() {
            let json_data = json!({
                "page": 2,
                "page_size": 20,
                "category_id": 5,
                "name": "苹果",
                "is_disabled": false
            });

            let params: ProductListQueryParams = serde_json::from_value(json_data).unwrap();
            assert_eq!(params.page, Some(2));
            assert_eq!(params.page_size, Some(20));
            assert_eq!(params.category_id, Some(5));
            assert_eq!(params.name, Some("苹果".to_string()));
            assert_eq!(params.is_disabled, Some(false));
        }

        #[test]
        fn test_optional_params() {
            let json_data = json!({});
            let params: ProductListQueryParams = serde_json::from_value(json_data).unwrap();
            assert!(params.page.is_none());
            assert!(params.page_size.is_none());
            assert!(params.category_id.is_none());
            assert!(params.name.is_none());
            assert!(params.is_disabled.is_none());
        }
    }

    // Tests for ProductListDTO
    mod product_list_dto_tests {
        use super::*;

        #[test]
        fn test_product_list_serialization() {
            let dto = ProductListDTO {
                product_code: VALID_PRODUCT_CODE.to_string(),
                name: VALID_PRODUCT_NAME.to_string(),
                category_level1_id: VALID_CATEGORY_ID,
                category: "水果".to_string(),
                avg_price: create_decimal("6.00"),
                min_price: create_decimal("5.00"),
                max_price: create_decimal("7.00"),
                unit: VALID_UNIT.to_string(),
                image: Some("apple.jpg".to_string()),
                is_disabled: false,
                discount_rate: create_decimal("0.10"),
                min_order_quantity: create_decimal("1.0"),
            };

            let json = serde_json::to_value(&dto).unwrap();

            // Test field renaming
            assert!(json.get("id").is_some());
            assert!(json.get("categoryId").is_some());
            assert!(json.get("price").is_some());
            assert!(json.get("minPrice").is_some());
            assert!(json.get("maxPrice").is_some());
            assert!(json.get("isDisabled").is_some());
            assert!(json.get("discountRate").is_some());
            assert!(json.get("minOrderQuantity").is_some());

            // Test original field names are not present
            assert!(json.get("product_code").is_none());
            assert!(json.get("category_level1_id").is_none());
            assert!(json.get("avg_price").is_none());
            assert!(json.get("min_price").is_none());
            assert!(json.get("max_price").is_none());
            assert!(json.get("is_disabled").is_none());
            assert!(json.get("discount_rate").is_none());
            assert!(json.get("min_order_quantity").is_none());

            assert_eq!(json["id"], VALID_PRODUCT_CODE);
            assert_eq!(json["name"], VALID_PRODUCT_NAME);
            assert_eq!(json["categoryId"], VALID_CATEGORY_ID);
            assert_eq!(json["price"], "6.00");
            assert_eq!(json["isDisabled"], false);
        }
    }

    // Tests for ProductOverviewDTO and ProductOverviewResponse
    mod product_overview_tests {
        use super::*;

        #[test]
        fn test_product_overview_dto() {
            let dto = ProductOverviewDTO {
                product_code: VALID_PRODUCT_CODE.to_string(),
                name: VALID_PRODUCT_NAME.to_string(),
                unit: VALID_UNIT.to_string(),
                description: Some("简单描述".to_string()),
                is_disabled: false,
            };

            let json = serde_json::to_value(&dto).unwrap();

            // Test field renaming
            assert!(json.get("id").is_some());
            assert!(json.get("isDisabled").is_some());

            // Test original field names are not present
            assert!(json.get("product_code").is_none());
            assert!(json.get("is_disabled").is_none());

            assert_eq!(json["id"], VALID_PRODUCT_CODE);
            assert_eq!(json["name"], VALID_PRODUCT_NAME);
            assert_eq!(json["isDisabled"], false);
        }

        #[test]
        fn test_product_overview_response() {
            let products = vec![
                ProductOverviewDTO {
                    product_code: "P001".to_string(),
                    name: "苹果".to_string(),
                    unit: "kg".to_string(),
                    description: Some("红苹果".to_string()),
                    is_disabled: false,
                },
                ProductOverviewDTO {
                    product_code: "P002".to_string(),
                    name: "香蕉".to_string(),
                    unit: "kg".to_string(),
                    description: None,
                    is_disabled: true,
                },
            ];

            let response = ProductOverviewResponse {
                products,
                total: 2,
            };

            let json = serde_json::to_value(&response).unwrap();
            assert_eq!(json["total"], 2);
            assert_eq!(json["products"].as_array().unwrap().len(), 2);
            assert_eq!(json["products"][0]["id"], "P001");
            assert_eq!(json["products"][1]["id"], "P002");
            assert_eq!(json["products"][0]["isDisabled"], false);
            assert_eq!(json["products"][1]["isDisabled"], true);
        }
    }

    // Tests for PriceQueryParams validation (inherited from price.rs)
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
        fn test_invalid_date_format() {
            let params = PriceQueryParams::builder()
                .date(INVALID_DATE_FORMAT)
                .build();
            assert!(params.validate().is_err());
        }

        #[test]
        fn test_invalid_category_ids() {
            let params = PriceQueryParams::builder()
                .category1(INVALID_CATEGORY_ID)
                .build();
            assert!(params.validate().is_err());

            let params = PriceQueryParams::builder()
                .category3(INVALID_CATEGORY_ID)
                .build();
            assert!(params.validate().is_err());
        }

        #[test]
        fn test_invalid_product_names() {
            // Test empty string
            let params = PriceQueryParams::builder()
                .name(EMPTY_PRODUCT_NAME)
                .build();
            assert!(params.validate().is_err());

            // Test too long name
            let params = PriceQueryParams::builder()
                .name(LONG_PRODUCT_NAME)
                .build();
            assert!(params.validate().is_err());
        }

        #[test]
        fn test_validation_error_messages() {
            // Test date format error
            let params = PriceQueryParams::builder()
                .date(INVALID_DATE_FORMAT)
                .build();
            let error = params.validate().unwrap_err();
            assert!(error.contains("Invalid date format"));

            // Test category error
            let params = PriceQueryParams::builder()
                .category1(INVALID_CATEGORY_ID)
                .build();
            let error = params.validate().unwrap_err();
            assert!(error.contains("Category1 ID must be positive"));

            // Test name error
            let params = PriceQueryParams::builder()
                .name(EMPTY_PRODUCT_NAME)
                .build();
            let error = params.validate().unwrap_err();
            assert!(error.contains("Product name cannot be empty"));
        }
    }

    // Integration tests
    mod integration_tests {
        use super::*;

        #[test]
        fn test_complete_product_workflow() {
            // Test a complete product workflow from creation to listing
            let detail = create_test_product_detail_dto();
            let processing_fee = create_test_processing_fee_dto();

            // Create product detail response
            let detail_response = ProductDetailResponse {
                product: detail,
                processing_fees: Some(vec![processing_fee]),
            };

            // Serialize and deserialize
            let json = serde_json::to_string(&detail_response).unwrap();
            let deserialized: ProductDetailResponse = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.product.name, VALID_PRODUCT_NAME);
            assert_eq!(deserialized.processing_fees.as_ref().unwrap().len(), 1);
        }

        #[test]
        fn test_price_comparison_workflow() {
            let comparison = create_test_product_daily_price_comparison();

            // Serialize to JSON
            let json = serde_json::to_string(&comparison).unwrap();
            let deserialized: ProductDailyPriceComparisonDTO = serde_json::from_str(&json).unwrap();

            // Verify price data integrity
            assert_eq!(comparison.min_price, deserialized.min_price);
            assert_eq!(comparison.avg_price, deserialized.avg_price);
            assert_eq!(comparison.max_price, deserialized.max_price);
            assert_eq!(comparison.yesterday_min_price, deserialized.yesterday_min_price);
            assert_eq!(comparison.yesterday_avg_price, deserialized.yesterday_avg_price);
            assert_eq!(comparison.yesterday_max_price, deserialized.yesterday_max_price);
        }

        #[test]
        fn test_product_update_partial_fields() {
            // Test updating only specific fields
            let update_request = UpdateProductRequestDTO {
                name: Some("更新的产品名".to_string()),
                is_disabled: Some(true),
                processing_services: Some(vec![1, 2]),
                // All other fields None
                unit: None,
                product_description: None,
                brand: None,
                storage_conditions: None,
                shelf_life: None,
                pricing_method: None,
                special_notes: None,
                tips: None,
                spec: None,
                tax_rate: None,
                min_order_quantity: None,
            };

            let json = serde_json::to_string(&update_request).unwrap();
            let deserialized: UpdateProductRequestDTO = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.name, Some("更新的产品名".to_string()));
            assert_eq!(deserialized.is_disabled, Some(true));
            assert_eq!(deserialized.processing_services, Some(vec![1, 2]));
            assert!(deserialized.unit.is_none());
            assert!(deserialized.brand.is_none());
        }

        #[test]
        fn test_price_status_statistics() {
            let status = PriceStatusDTO {
                category_id: 1,
                category_name: "测试类别".to_string(),
                total_products: 50,
                products_without_price: 5,
                products_pending: 10,
                products_approved: 15,
                products_published: 18,
                products_rejected: 2,
            };

            // Verify statistics integrity
            let calculated_total = status.products_without_price 
                + status.products_pending 
                + status.products_approved 
                + status.products_published 
                + status.products_rejected;
            
            assert_eq!(calculated_total, status.total_products);

            // Test serialization
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: PriceStatusDTO = serde_json::from_str(&json).unwrap();
            assert_eq!(status.total_products, deserialized.total_products);
        }

        #[test]
        fn test_edge_cases_and_boundary_values() {
            // Test with zero values
            let price_dto = ProductPriceDTO {
                product_code: "P000".to_string(),
                level_one_category: "测试".to_string(),
                level_three_category: "测试".to_string(),
                product_name: "零价格产品".to_string(),
                min_price: create_decimal("0.00"),
                min_price_change: create_decimal("0.00"),
                avg_price: create_decimal("0.00"),
                avg_price_change: create_decimal("0.00"),
                max_price: create_decimal("0.00"),
                max_price_change: create_decimal("0.00"),
                unit: "pcs".to_string(),
                price_status: "DRAFT".to_string(),
            };

            let json = serde_json::to_string(&price_dto).unwrap();
            let deserialized: ProductPriceDTO = serde_json::from_str(&json).unwrap();
            assert_eq!(price_dto.min_price, deserialized.min_price);
            assert_eq!(price_dto.avg_price, deserialized.avg_price);
            assert_eq!(price_dto.max_price, deserialized.max_price);

            // Test with negative price changes
            assert_eq!(price_dto.min_price_change, create_decimal("0.00"));
            assert_eq!(price_dto.avg_price_change, create_decimal("0.00"));
            assert_eq!(price_dto.max_price_change, create_decimal("0.00"));
        }
    }
}
