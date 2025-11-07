use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Claims {
    pub(crate) tenant_type: String,
    pub(crate) tenant_name: String,
    pub(crate) tenant_hash: String,
    pub(crate) username: String,
    pub(crate) real_name: String,
    pub(crate) roles: Vec<String>,
    pub(crate) exp: usize,
    pub(crate) is_super_admin: bool,
}
