use crate::{
    dto::validate_phone,
    utils::validator::{validate_email, validate_password},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use validator::Validate;

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Validate, FromRow, utoipa::ToSchema,
)]
pub struct UserCreateDto {
    #[validate(length(min = 2, max = 16))]
    pub name: String,

    #[validate(length(min = 4, max = 16))]
    pub username: String,

    #[validate(custom(function = validate_password))]
    pub password: String,

    #[validate(custom(function = validate_email))]
    pub email: String,

    #[validate(custom(function = validate_phone))]
    pub phone: String,

    #[validate(length(min = 2, max = 32))]
    pub role: String,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Validate, FromRow, utoipa::ToSchema,
)]
pub struct UserResponseDto {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub role: Option<String>,

    #[serde(rename = "tenantName")]
    pub tenant_name: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,

    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Validate, FromRow, utoipa::ToSchema,
)]
pub(crate) struct UserUpdateDto {
    #[validate(length(min = 2, max = 16))]
    pub(crate) name: String,

    #[validate(custom(function = validate_email))]
    pub(crate) email: String,

    #[validate(custom(function = validate_phone))]
    pub(crate) phone: String,
}
