use crate::dto::financial::FinancialResponseDto;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

fn validate_tenant_type(tenant_type: &str) -> Result<(), validator::ValidationError> {
    match tenant_type.to_uppercase().as_str() {
        "PROVIDER" | "MARKET" | "CUSTOMER" => Ok(()),
        _ => {
            let mut error = validator::ValidationError::new("invalid_tenant_type");
            error.message = Some("Invalid tenant type".to_string().into());
            Err(error)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub struct TenantAddressUpdate {
    #[validate(length(min = 6, max = 255))]
    pub address: Option<String>,

    #[validate(length(min = 2, max = 255))]
    pub business_scope: Option<String>,
}

#[derive(
    Debug, Serialize, Deserialize, Validate, Clone, FromRow, PartialEq, Eq, utoipa::ToSchema,
)]
pub struct Provider {
    pub id: i32,
    pub name: String,

    #[serde(rename = "businessScope")]
    pub business_scope: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct QueryTenantByType {
    #[serde(rename = "tenantType")]
    pub tenant_type: Option<String>,
}

#[derive(
    Debug, Serialize, Deserialize, FromRow, Validate, Clone, PartialEq, Eq, utoipa::ToSchema,
)]
pub struct BaseTenantDTO {
    #[serde(skip_deserializing)]
    pub id: i32,

    #[validate(length(min = 6, max = 255))]
    pub name: String,

    #[validate(length(min = 6, max = 64))]
    pub address: String,

    #[validate(length(max = 12))]
    #[serde(rename = "tenantType")]
    pub tenant_type: String,

    #[serde(skip_deserializing)]
    #[serde(rename = "nameHash")]
    pub name_hash: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "businessScope")]
    pub business_scope: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "licenseImage")]
    pub license_image: Option<String>,

    pub status: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "verifiedAt")]
    pub verified_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub struct TenantCreateDTO {
    #[validate(length(min = 6, max = 255))]
    pub name: String,

    #[validate(length(min = 6, max = 64))]
    pub address: String,

    #[validate(length(max = 16), custom(function = validate_tenant_type))]
    #[serde(rename = "tenantType")]
    pub tenant_type: String,

    #[serde(rename = "businessScope")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 128))]
    pub business_scope: Option<String>,

    #[serde(rename = "licenseImage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 2, max = 255))]
    pub license_image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Validate, Clone, PartialEq, Eq)]
pub struct TenantDetailDTO {
    #[serde(skip_deserializing)]
    pub id: i32,

    #[validate(length(min = 6, max = 255))]
    pub name: String,

    #[validate(length(min = 6, max = 64))]
    pub address: String,

    #[validate(length(max = 12))]
    #[serde(rename = "tenantType")]
    pub tenant_type: String,

    #[serde(skip_deserializing)]
    #[serde(rename = "nameHash")]
    pub name_hash: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "businessScope")]
    pub business_scope: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "licenseImage")]
    pub license_image: Option<String>,

    pub status: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "verifiedAt")]
    pub verified_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "financial")]
    pub financial: Option<FinancialResponseDto>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_address_update_validation() {
        // 有效的数据
        let valid_update = TenantAddressUpdate {
            address: Some("北京市朝阳区建国路88号".to_string()),
            business_scope: Some("食品零售".to_string()),
        };
        assert!(valid_update.validate().is_ok());

        // 地址太短
        let short_address = TenantAddressUpdate {
            address: Some("北京".to_string()),
            business_scope: Some("食品零售".to_string()),
        };
        assert!(short_address.validate().is_err());

        // 业务范围太短
        let short_scope = TenantAddressUpdate {
            address: Some("北京市朝阳区建国路88号".to_string()),
            business_scope: Some("食".to_string()),
        };
        assert!(short_scope.validate().is_err());

        // 地址为None是有效的
        let no_address = TenantAddressUpdate {
            address: None,
            business_scope: Some("食品零售".to_string()),
        };
        assert!(no_address.validate().is_ok());

        // 业务范围为None是有效的
        let no_scope = TenantAddressUpdate {
            address: Some("北京市朝阳区建国路88号".to_string()),
            business_scope: None,
        };
        assert!(no_scope.validate().is_ok());

        // 超长地址
        let long_address = TenantAddressUpdate {
            address: Some("a".repeat(256)),
            business_scope: Some("食品零售".to_string()),
        };
        assert!(long_address.validate().is_err());
    }

    #[test]
    fn test_tenant_validation() {
        // 有效的TenantDTO
        let valid_tenant = BaseTenantDTO {
            id: 1,
            name: "北京市朝阳区新世界超市".to_string(),
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            name_hash: "abc123".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: None,
            status: "ACTIVE".to_string(),
            created_at: None,
            updated_at: None,
            verified_at: None,
            deleted_at: None,
        };
        assert!(valid_tenant.validate().is_ok());

        // 名称太短
        let short_name_tenant = BaseTenantDTO {
            id: 1,
            name: "超市".to_string(),
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            name_hash: "abc123".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: None,
            status: "ACTIVE".to_string(),
            created_at: None,
            updated_at: None,
            verified_at: None,
            deleted_at: None,
        };
        assert!(short_name_tenant.validate().is_err());

        // 地址太短
        let short_address_tenant = BaseTenantDTO {
            id: 1,
            name: "北京市朝阳区新世界超市".to_string(),
            address: "北京".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            name_hash: "abc123".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: None,
            status: "ACTIVE".to_string(),
            created_at: None,
            updated_at: None,
            verified_at: None,
            deleted_at: None,
        };
        assert!(short_address_tenant.validate().is_err());

        // 超长名称
        let long_name_tenant = BaseTenantDTO {
            id: 1,
            name: "a".repeat(256),
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            name_hash: "abc123".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: None,
            status: "ACTIVE".to_string(),
            created_at: None,
            updated_at: None,
            verified_at: None,
            deleted_at: None,
        };
        assert!(long_name_tenant.validate().is_err());

        // 测试tenant_type长度限制
        let long_type_tenant = BaseTenantDTO {
            id: 1,
            name: "北京市朝阳区新世界超市".to_string(),
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMERCUSTOMERCUSTOMER".to_string(), // 超过12个字符
            name_hash: "abc123".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: None,
            status: "ACTIVE".to_string(),
            created_at: None,
            updated_at: None,
            verified_at: None,
            deleted_at: None,
        };
        assert!(long_type_tenant.validate().is_err());
    }

    #[test]
    fn test_tenant_create_dto_validation() {
        // 有效的数据
        let valid_dto = TenantCreateDTO {
            name: "北京市朝阳区新世界超市".to_string(),
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: Some("http://example.com/image.jpg".to_string()),
        };
        assert!(valid_dto.validate().is_ok());

        // 名称太短
        let short_name_dto = TenantCreateDTO {
            name: "超市".to_string(), // 少于6个字符
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: Some("http://example.com/image.jpg".to_string()),
        };
        assert!(short_name_dto.validate().is_err());

        // 地址太短
        let short_address_dto = TenantCreateDTO {
            name: "北京市朝阳区新世界超市".to_string(),
            address: "北京".to_string(), // 少于6个字符
            tenant_type: "CUSTOMER".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: Some("http://example.com/image.jpg".to_string()),
        };
        assert!(short_address_dto.validate().is_err());

        // 地址太长
        let long_address_dto = TenantCreateDTO {
            name: "北京市朝阳区新世界超市".to_string(),
            address: "a".repeat(65), // 超过64个字符
            tenant_type: "CUSTOMER".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: Some("http://example.com/image.jpg".to_string()),
        };
        assert!(long_address_dto.validate().is_err());

        // 名称太长
        let long_name_dto = TenantCreateDTO {
            name: "a".repeat(256), // 超过255个字符
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: Some("http://example.com/image.jpg".to_string()),
        };
        assert!(long_name_dto.validate().is_err());

        // tenant_type太长
        let long_type_dto = TenantCreateDTO {
            name: "北京市朝阳区新世界超市".to_string(),
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMERCUSTOMER".to_string(), // 超过12个字符
            business_scope: Some("食品零售".to_string()),
            license_image: Some("http://example.com/image.jpg".to_string()),
        };
        assert!(long_type_dto.validate().is_err());

        // business_scope太短
        let short_scope_dto = TenantCreateDTO {
            name: "北京市朝阳区新世界超市".to_string(),
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            business_scope: Some("食".to_string()), // 少于2个字符
            license_image: Some("http://example.com/image.jpg".to_string()),
        };
        assert!(short_scope_dto.validate().is_err());

        // license_image太短
        let short_license_dto = TenantCreateDTO {
            name: "北京市朝阳区新世界超市".to_string(),
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            business_scope: Some("食品零售".to_string()),
            license_image: Some("a".to_string()), // 少于2个字符
        };
        assert!(short_license_dto.validate().is_err());

        // 可选字段为None
        let optional_none_dto = TenantCreateDTO {
            name: "北京市朝阳区新世界超市".to_string(),
            address: "北京市朝阳区建国路88号".to_string(),
            tenant_type: "CUSTOMER".to_string(),
            business_scope: None,
            license_image: None,
        };
        assert!(optional_none_dto.validate().is_ok());
    }
}
