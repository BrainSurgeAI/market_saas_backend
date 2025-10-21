use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Option<i32>,

    pub username: String,

    #[serde(skip_deserializing)]
    pub password_hash: String,

    #[serde(skip_serializing)]
    pub password: String,

    pub email: Option<String>,

    pub phone: Option<String>,

    #[serde(skip_deserializing)]
    pub tenant_id: Option<i32>,

    pub roles: Vec<i32>,

    #[serde(skip_serializing)]
    pub is_super_admin: bool,
}
