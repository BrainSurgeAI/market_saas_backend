use chrono::Utc;
use rand::Rng;
use std::hash::{DefaultHasher, Hash, Hasher};
use tracing::debug;

use crate::common::AppError;

pub(crate) mod category_traits;
pub(crate) mod delivery_staff_traits;
pub(crate) mod discount_traits;
pub(crate) mod order_traits;
pub(crate) mod price_traits;
pub(crate) mod product_traits;
pub(crate) mod reconciliation_statement_traits;
pub(crate) mod role_traits;
pub(crate) mod superadmin_traits;
pub(crate) mod system_log_repo;
pub(crate) mod tenants_trait;
pub(crate) mod user_traits;


#[macro_export]
macro_rules! map_db_err {
    ($msg:expr) => {
        |e| {
            error!(concat!($msg, ": {:#?}"), e);
            AppError::Database(e)
        }
    };
}

pub(crate) mod my_sql_repository {
    use sqlx::MySqlPool;

    #[derive(Debug, Clone)]
    pub(crate) struct MySqlRepository {
        pub(crate) pool: MySqlPool,
    }

    impl MySqlRepository {
        pub(crate) fn new(pool: MySqlPool) -> Self {
            Self { pool }
        }
    }
}

/// Generate a hash for a given tenant name
///
/// This function generates a hash for a given tenant name.
/// Because we don't use uuid, the hash is used to identify the tenant uniquely.
///
/// # Parameters
///
/// * `name`: The name of the tenant.
///
/// # Returns
///
/// A string representing the generated hash.
///
/// # Errors
///
/// Returns an `AppError` if the tenant name is empty.
pub(super) fn generate_tenant_name_hash(name: &str) -> Result<String, AppError> {
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

pub(crate) enum CodeType {
    Order,
    ReconciliationStatement,
}

/// Generate a order code or reconciliation statement code for a given code type
///
/// This function generates a order code or reconciliation statement code for a given code type.
///
/// # Parameters
///
/// * `code_type`: The type of code to generate. Can be `Order` or `ReconciliationStatement`.
///
/// # Returns
///
/// A string representing the generated code. The format is `{code_type}-{timestamp}-{4-letter-random-suffix}`.
pub(crate) fn generate_code(code_type: CodeType) -> String {
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
