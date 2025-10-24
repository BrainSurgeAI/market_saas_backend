use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Validate, Serialize, Deserialize)]
pub struct Role {
    #[validate(range(min = 1))]
    pub id: i32,

    #[validate(length(min = 4, max = 32))]
    pub name: String,

    #[serde(rename = "tenantType")]
    #[validate(length(min = 4, max = 16))]
    pub tenant_type: Option<String>,

    #[serde(rename = "aliasName")]
    #[validate(length(min = 4, max = 32))]
    pub alias_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Validate, Serialize, Deserialize)]
pub struct PermissionResponseDto {
    pub id: i32,
    pub name: String,
    pub cname: Option<String>,
    pub description: Option<String>,

    #[serde(rename = "selfOnly")]
    pub self_only: Option<i8>,

    #[serde(rename = "pathPattern")]
    pub path_pattern: Option<String>,

    #[serde(rename = "httpMethod")]
    pub http_method: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Eq, FromRow, Validate, Serialize, Deserialize)]
pub struct PermissionCreateDto {
    #[validate(length(min = 4, max = 32))]
    pub name: String,

    #[validate(length(min = 4, max = 16))]
    pub cname: Option<String>,

    #[validate(length(min = 4, max = 32))]
    pub description: Option<String>,

    #[serde(rename = "selfOnly")]
    pub self_only: Option<i8>,

    #[serde(rename = "pathPattern")]
    #[validate(length(min = 4, max = 64))]
    pub path_pattern: Option<String>,

    #[serde(rename = "httpMethod")]
    #[validate(length(min = 3, max = 8))]
    pub http_method: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Validate, Serialize, Deserialize)]
pub struct UpdateRolePermissionDTO {
    #[serde(rename = "permissionIds")]
    pub permissions: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: i32,
    pub title: String,
    pub url: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<i8>,
    pub parent_id: Option<i32>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct MenuConfig {
    pub id: i32,
    pub title: String,
    pub url: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub items: Option<Vec<MenuConfig>>,
}
