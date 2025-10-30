use chrono::Utc;
use rand::Rng;

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
pub(super) fn generate_tenant_name_hash() -> String {
    use uuid::Uuid;
    const CHARSET: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let mut code = String::with_capacity(8);
    for &b in &bytes[..8] {
        code.push(CHARSET[(b as usize) % CHARSET.len()] as char);
    }
    code
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
