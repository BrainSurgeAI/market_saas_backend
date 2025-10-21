use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub tenant_type: String,
    pub tenant_name: String,
    pub tenant_hash: String,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub exp: usize,
    pub is_super_admin: bool,
}
