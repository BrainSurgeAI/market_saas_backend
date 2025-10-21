use chrono::Utc;
use rand::Rng;
use std::fmt;
use std::hash::{DefaultHasher, Hash, Hasher};
use tracing::debug;

use crate::common::AppError;

pub mod category_traits;
pub mod delivery_staff_traits;
pub mod discount_traits;
pub mod order_traits;
pub mod price_traits;
pub mod product_traits;
pub mod reconciliation_statement_traits;
pub mod role_traits;
pub mod superadmin_traits;
pub mod system_log_repo;
pub mod tenants_trait;
pub mod user_traits;
pub mod my_sql_repository {
    use sqlx::MySqlPool;

    #[derive(Debug, Clone)]
    pub struct MySqlRepository {
        pub(crate) pool: MySqlPool,
    }

    impl MySqlRepository {
        pub fn new(pool: MySqlPool) -> Self {
            Self { pool }
        }
    }
}

pub enum TenantType {
    Customer,
    Provider,
    Market,
}

impl fmt::Display for TenantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TenantType::Customer => write!(f, "CUSTOMER"),
            TenantType::Provider => write!(f, "PROVIDER"),
            TenantType::Market => write!(f, "MARKET"),
        }
    }
}

impl From<&str> for TenantType {
    fn from(s: &str) -> Self {
        match s {
            "CUSTOMER" => TenantType::Customer,
            "PROVIDER" => TenantType::Provider,
            "MARKET" => TenantType::Market,
            _ => panic!("未知的租户类型: {}", s),
        }
    }
}

pub fn tenant_name_hash(name: &str) -> Result<String, AppError> {
    debug!("Hashing tenant name: {}", name);

    if name.trim().is_empty() {
        return Err(AppError::Validation(
            "Tenant name cannot be empty".to_string(),
        ));
    }

    // 1. 计算输入字符串的哈希值
    let mut hasher = DefaultHasher::new();
    // 使用完整的名字来计算哈希，确保唯一性
    name.hash(&mut hasher);
    let hash = hasher.finish();

    // 2. 定义字符集（移除容易混淆的字符如0o1l）
    const CHARSET: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";

    // 3. 生成8位字符
    let mut result = String::with_capacity(8);

    // 确保第一位是字母
    let first_idx = (hash % 24) as usize; // 24是字母的数量
    result.push(CHARSET[first_idx] as char);

    // 生成剩余7位
    let mut remaining_hash = hash >> 6; // 移位操作以使用哈希值的不同部分
    for _ in 0..7 {
        let idx = (remaining_hash % (CHARSET.len() as u64)) as usize;
        result.push(CHARSET[idx] as char);
        remaining_hash /= CHARSET.len() as u64;
    }

    Ok(result)
}

pub enum CodeType {
    Order,
    ReconciliationStatement,
}

/// 生成订单编号，格式为 ODR-时间戳-4位随机字母
pub fn generate_code(code_type: CodeType) -> String {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S%3f");
    let random_suffix: String = rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .map(char::from)
        .filter(|c| c.is_alphabetic())
        .take(4)
        .collect::<String>()
        .to_uppercase();

    match code_type {
        CodeType::Order => format!("ODR-{}-{}", timestamp, random_suffix),
        CodeType::ReconciliationStatement => format!("RS-{}-{}", timestamp, random_suffix),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_order_code() {
        let code = generate_code(CodeType::Order);
        assert!(code.starts_with("ODR-"));
        assert!(code.len() > 20);
        assert!(code.len() < 30);
    }

    #[test]
    fn test_generate_reconciliation_statement_code() {
        let code = generate_code(CodeType::ReconciliationStatement);
        assert!(code.starts_with("RS-"));
        assert!(code.len() > 20);
        assert!(code.len() < 30);
    }
}
