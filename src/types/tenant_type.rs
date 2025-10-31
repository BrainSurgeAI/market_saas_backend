// use serde::{Deserialize, Serialize};

// ... existing code ...
// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, utoipa::ToSchema)]
// pub enum TenantStatus {
//     Pending,
//     Submitted,
//     Active,
//     Rejected,
// }

// impl sqlx::Type<sqlx::MySql> for TenantStatus {
//     fn type_info() -> sqlx::mysql::MySqlTypeInfo {
//         <str as sqlx::Type<sqlx::MySql>>::type_info()
//     }
// }

// impl<'r> sqlx::Decode<'r, sqlx::MySql> for TenantStatus {
//     fn decode(value: sqlx::mysql::MySqlValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
//         let str_value = <&str as sqlx::Decode<sqlx::MySql>>::decode(value)?;
//         match str_value {
//             "PENDING" => Ok(TenantStatus::Pending),
//             "SUBMITTED" => Ok(TenantStatus::Submitted),
//             "ACTIVE" => Ok(TenantStatus::Active),
//             "REJECTED" => Ok(TenantStatus::Rejected),
//             _ => Err("Invalid tenant status".into()),
//         }
//     }
// }
// ... existing code ...
