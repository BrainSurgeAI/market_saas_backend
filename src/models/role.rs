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
pub struct Permission {
    #[validate(range(min = 1))]
    pub id: i32,

    #[validate(length(min = 4, max = 32))]
    pub name: String,

    #[validate(length(min = 4, max = 16))]
    pub cname: Option<String>,

    #[validate(length(min = 4, max = 32))]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Validate, Serialize, Deserialize)]
pub struct UpdateRolePermissionDTO {
    #[serde(rename = "permissionIds")]
    pub permissions: Vec<i32>,
}
