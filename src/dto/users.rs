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

// Constructor methods for better ergonomics
impl UserCreateDto {
    pub fn new(
        name: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        email: impl Into<String>,
        phone: impl Into<String>,
        role: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            username: username.into(),
            password: password.into(),
            email: email.into(),
            phone: phone.into(),
            role: role.into(),
        }
    }

    /// Builder pattern for flexible construction
    #[cfg(test)]
    pub fn builder() -> UserCreateDtoBuilder {
        UserCreateDtoBuilder::default()
    }

    /// Check if user has admin role
    pub fn is_admin(&self) -> bool {
        self.role.to_lowercase() == "admin"
    }

    /// Check if user has manager role
    pub fn is_manager(&self) -> bool {
        self.role.to_lowercase() == "manager"
    }

    /// Check if username meets minimum requirements
    pub fn has_valid_username_length(&self) -> bool {
        self.username.len() >= 4 && self.username.len() <= 16
    }

    /// Check if name meets requirements
    pub fn has_valid_name_length(&self) -> bool {
        self.name.len() >= 2 && self.name.len() <= 16
    }

    /// Check if role meets requirements
    pub fn has_valid_role_length(&self) -> bool {
        self.role.len() >= 2 && self.role.len() <= 32
    }
}

impl UserResponseDto {
    pub fn new(
        id: i32,
        name: impl Into<String>,
        username: impl Into<String>,
        tenant_name: impl Into<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            username: username.into(),
            email: None,
            phone: None,
            role: None,
            tenant_name: tenant_name.into(),
            created_at,
            updated_at,
            deleted_at: None,
        }
    }

    /// Check if user is deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// Get user creation age in days
    pub fn creation_age_days(&self) -> i64 {
        let now = Utc::now();
        (now - self.created_at).num_days()
    }

    /// Check if user was recently updated (within last 24 hours)
    pub fn recently_updated(&self) -> bool {
        let now = Utc::now();
        (now - self.updated_at).num_hours() < 24
    }

    /// Get display name with fallback
    pub fn display_name(&self) -> &str {
        if !self.name.is_empty() {
            &self.name
        } else {
            &self.username
        }
    }
}

impl UserUpdateDto {
    pub fn new(
        name: impl Into<String>,
        email: impl Into<String>,
        phone: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            phone: phone.into(),
        }
    }

    /// Check if name meets requirements
    pub fn has_valid_name_length(&self) -> bool {
        self.name.len() >= 2 && self.name.len() <= 16
    }
}

// Builder pattern for UserCreateDto
#[cfg(test)]
#[derive(Default)]
pub struct UserCreateDtoBuilder {
    dto: UserCreateDto,
}

#[cfg(test)]
impl UserCreateDtoBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.dto.name = name.into();
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.dto.username = username.into();
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.dto.password = password.into();
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.dto.email = email.into();
        self
    }

    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.dto.phone = phone.into();
        self
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.dto.role = role.into();
        self
    }

    pub fn build(self) -> UserCreateDto {
        self.dto
    }
}

#[cfg(test)]
impl Default for UserCreateDto {
    fn default() -> Self {
        Self {
            name: String::new(),
            username: String::new(),
            password: String::new(),
            email: String::new(),
            phone: String::new(),
            role: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    // Test constants for better maintainability
    const VALID_NAME: &str = "张三";
    const VALID_USERNAME: &str = "zhangsan";
    const VALID_PASSWORD: &str = "Password123!";
    const VALID_EMAIL: &str = "test@example.com";
    const VALID_PHONE: &str = "13812345678";
    const VALID_ROLE: &str = "admin";
    const VALID_TENANT_NAME: &str = "测试租户";

    const MIN_NAME: &str = "李四"; // 2 characters, valid
    const MIN_USERNAME: &str = "test";
    const MIN_ROLE: &str = "管理"; // 2 characters, valid

    const INVALID_PASSWORD: &str = "123";
    const INVALID_EMAIL: &str = "invalid-email";
    const INVALID_PHONE: &str = "123";

    const TOO_SHORT_NAME: &str = "X"; // 1 character, invalid
    const TOO_LONG_NAME: &str = "这是一个非常长的名字确实超过了十六个字符的限制测试"; // 22 chars, invalid
    const TOO_SHORT_USERNAME: &str = "abc"; // 3 characters, invalid
    const TOO_LONG_USERNAME: &str = "verylongusernamethatexceedslimit";
    const TOO_SHORT_ROLE: &str = "X"; // 1 character, invalid
    const TOO_LONG_ROLE: &str = "这是一个非常长的角色名称确实超过了三十二个字符的限制应该会失败的测试用例数据"; // 35 chars, invalid

    // Helper functions to create test data
    fn create_valid_user_create_dto() -> UserCreateDto {
        UserCreateDto::new(
            VALID_NAME,
            VALID_USERNAME,
            VALID_PASSWORD,
            VALID_EMAIL,
            VALID_PHONE,
            VALID_ROLE,
        )
    }

    fn create_valid_user_response_dto() -> UserResponseDto {
        UserResponseDto {
            id: 1,
            name: VALID_NAME.to_string(),
            username: VALID_USERNAME.to_string(),
            email: Some(VALID_EMAIL.to_string()),
            phone: Some(VALID_PHONE.to_string()),
            role: Some(VALID_ROLE.to_string()),
            tenant_name: VALID_TENANT_NAME.to_string(),
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
            deleted_at: None,
        }
    }

    fn create_valid_user_update_dto() -> UserUpdateDto {
        UserUpdateDto::new(VALID_NAME, VALID_EMAIL, VALID_PHONE)
    }

    // Helper function to check validation errors
    fn assert_validation_error(result: Result<(), validator::ValidationErrors>, field: &str) {
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key(field));
    }

    // Tests for UserCreateDto
    mod user_create_dto_tests {
        use super::*;

        #[test]
        fn test_valid_user_creation() {
            let dto = create_valid_user_create_dto();
            assert!(dto.validate().is_ok());
            assert_eq!(dto.name, VALID_NAME);
            assert_eq!(dto.username, VALID_USERNAME);
            assert_eq!(dto.email, VALID_EMAIL);
            assert_eq!(dto.phone, VALID_PHONE);
            assert_eq!(dto.role, VALID_ROLE);
        }

        #[test]
        fn test_builder_pattern() {
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();

            assert!(dto.validate().is_ok());
            assert_eq!(dto.name, VALID_NAME);
            assert_eq!(dto.username, VALID_USERNAME);
        }

        #[test]
        fn test_serialization() {
            let dto = create_valid_user_create_dto();
            let json = serde_json::to_value(&dto).unwrap();

            assert_eq!(json["name"], VALID_NAME);
            assert_eq!(json["username"], VALID_USERNAME);
            assert_eq!(json["password"], VALID_PASSWORD);
            assert_eq!(json["email"], VALID_EMAIL);
            assert_eq!(json["phone"], VALID_PHONE);
            assert_eq!(json["role"], VALID_ROLE);
        }

        #[test]
        fn test_deserialization() {
            let json_data = json!({
                "name": "王五",
                "username": "wangwu",
                "password": "SecurePass123!",
                "email": "wangwu@example.com",
                "phone": "13987654321",
                "role": "user"
            });

            let dto: UserCreateDto = serde_json::from_value(json_data).unwrap();
            assert_eq!(dto.name, "王五");
            assert_eq!(dto.username, "wangwu");
            assert_eq!(dto.email, "wangwu@example.com");
            assert_eq!(dto.phone, "13987654321");
            assert_eq!(dto.role, "user");
        }

        #[test]
        fn test_name_length_validation() {
            // Valid name lengths
            let dto = UserCreateDto::builder()
                .name(MIN_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert!(dto.validate().is_ok());

            // Too short name
            let dto = UserCreateDto::builder()
                .name(TOO_SHORT_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert_validation_error(dto.validate(), "name");

            // Too long name
            let dto = UserCreateDto::builder()
                .name(TOO_LONG_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert_validation_error(dto.validate(), "name");
        }

        #[test]
        fn test_username_length_validation() {
            // Valid username lengths
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(MIN_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert!(dto.validate().is_ok());

            // Too short username
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(TOO_SHORT_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert_validation_error(dto.validate(), "username");

            // Too long username
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(TOO_LONG_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert_validation_error(dto.validate(), "username");
        }

        #[test]
        fn test_password_validation() {
            // Invalid password
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(INVALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert_validation_error(dto.validate(), "password");
        }

        #[test]
        fn test_email_validation() {
            // Invalid email
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(INVALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert_validation_error(dto.validate(), "email");
        }

        #[test]
        fn test_phone_validation() {
            // Invalid phone
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(INVALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert_validation_error(dto.validate(), "phone");
        }

        #[test]
        fn test_role_length_validation() {
            // Valid role lengths
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(MIN_ROLE)
                .build();
            assert!(dto.validate().is_ok());

            // Too short role
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(TOO_SHORT_ROLE)
                .build();
            assert_validation_error(dto.validate(), "role");

            // Too long role
            let dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(TOO_LONG_ROLE)
                .build();
            assert_validation_error(dto.validate(), "role");
        }

        #[test]
        fn test_business_logic_methods() {
            let admin_dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role("admin")
                .build();
            assert!(admin_dto.is_admin());
            assert!(!admin_dto.is_manager());

            let manager_dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role("Manager")
                .build();
            assert!(!manager_dto.is_admin());
            assert!(manager_dto.is_manager());

            let user_dto = UserCreateDto::builder()
                .name(VALID_NAME)
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role("user")
                .build();
            assert!(!user_dto.is_admin());
            assert!(!user_dto.is_manager());
        }

        #[test]
        fn test_validation_helper_methods() {
            let dto = create_valid_user_create_dto();
            assert!(dto.has_valid_username_length());
            assert!(dto.has_valid_name_length());
            assert!(dto.has_valid_role_length());

            let invalid_dto = UserCreateDto::builder()
                .name(TOO_SHORT_NAME)
                .username(TOO_SHORT_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(TOO_SHORT_ROLE)
                .build();
            assert!(!invalid_dto.has_valid_username_length());
            assert!(!invalid_dto.has_valid_name_length());
            assert!(!invalid_dto.has_valid_role_length());
        }

        #[test]
        fn test_clone_and_equality() {
            let dto1 = create_valid_user_create_dto();
            let dto2 = dto1.clone();
            assert_eq!(dto1, dto2);

            let dto3 = UserCreateDto::builder()
                .name("不同名字")
                .username(VALID_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(VALID_ROLE)
                .build();
            assert_ne!(dto1, dto3);
        }
    }

    // Tests for UserResponseDto
    mod user_response_dto_tests {
        use super::*;

        #[test]
        fn test_serialization_with_field_renaming() {
            let dto = create_valid_user_response_dto();
            let json = serde_json::to_value(&dto).unwrap();

            // Test field renaming
            assert!(json.get("tenantName").is_some());
            assert!(json.get("createdAt").is_some());
            assert!(json.get("updatedAt").is_some());
            assert!(json.get("deletedAt").is_some());

            // Test original field names are not present
            assert!(json.get("tenant_name").is_none());
            assert!(json.get("created_at").is_none());
            assert!(json.get("updated_at").is_none());
            assert!(json.get("deleted_at").is_none());

            // Test values
            assert_eq!(json["id"], 1);
            assert_eq!(json["name"], VALID_NAME);
            assert_eq!(json["username"], VALID_USERNAME);
            assert_eq!(json["email"], VALID_EMAIL);
            assert_eq!(json["phone"], VALID_PHONE);
            assert_eq!(json["role"], VALID_ROLE);
            assert_eq!(json["tenantName"], VALID_TENANT_NAME);
            assert!(json["deletedAt"].is_null());
        }

        #[test]
        fn test_deserialization() {
            let json_data = json!({
                "id": 2,
                "name": "李四",
                "username": "lisi",
                "email": "lisi@example.com",
                "phone": "13765432109",
                "role": "manager",
                "tenantName": "新租户",
                "createdAt": "2024-02-01T00:00:00Z",
                "updatedAt": "2024-02-02T00:00:00Z",
                "deletedAt": null
            });

            let dto: UserResponseDto = serde_json::from_value(json_data).unwrap();
            assert_eq!(dto.id, 2);
            assert_eq!(dto.name, "李四");
            assert_eq!(dto.username, "lisi");
            assert_eq!(dto.email, Some("lisi@example.com".to_string()));
            assert_eq!(dto.phone, Some("13765432109".to_string()));
            assert_eq!(dto.role, Some("manager".to_string()));
            assert_eq!(dto.tenant_name, "新租户");
            assert!(dto.deleted_at.is_none());
        }

        #[test]
        fn test_optional_fields() {
            let dto = UserResponseDto {
                id: 3,
                name: "测试用户".to_string(),
                username: "testuser".to_string(),
                email: None,
                phone: None,
                role: None,
                tenant_name: VALID_TENANT_NAME.to_string(),
                created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
                deleted_at: None,
            };

            let json = serde_json::to_value(&dto).unwrap();
            assert!(json["email"].is_null());
            assert!(json["phone"].is_null());
            assert!(json["role"].is_null());
            assert!(json["deletedAt"].is_null());
        }

        #[test]
        fn test_deleted_user() {
            let deleted_time = Utc.with_ymd_and_hms(2024, 1, 10, 0, 0, 0).unwrap();
            let dto = UserResponseDto {
                id: 4,
                name: "删除用户".to_string(),
                username: "deleteduser".to_string(),
                email: Some("deleted@example.com".to_string()),
                phone: Some("13800000000".to_string()),
                role: Some("user".to_string()),
                tenant_name: VALID_TENANT_NAME.to_string(),
                created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
                deleted_at: Some(deleted_time),
            };

            assert!(dto.is_deleted());
            assert!(!dto.is_active());

            let json = serde_json::to_value(&dto).unwrap();
            assert_eq!(json["deletedAt"], "2024-01-10T00:00:00Z");
        }

        #[test]
        fn test_business_logic_methods() {
            let dto = create_valid_user_response_dto();
            
            // Test user status
            assert!(!dto.is_deleted());
            assert!(dto.is_active());

            // Test creation age
            let age_days = dto.creation_age_days();
            assert!(age_days > 0); // Should be positive since created in past

            // Test recent update (this user was updated in 2024, so not recent)
            assert!(!dto.recently_updated());

            // Test display name
            assert_eq!(dto.display_name(), VALID_NAME);
        }

        #[test]
        fn test_display_name_fallback() {
            let dto = UserResponseDto {
                id: 5,
                name: String::new(), // Empty name
                username: "fallbackuser".to_string(),
                email: None,
                phone: None,
                role: None,
                tenant_name: VALID_TENANT_NAME.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };

            assert_eq!(dto.display_name(), "fallbackuser");
        }

        #[test]
        fn test_recently_updated() {
            let now = Utc::now();
            let dto = UserResponseDto {
                id: 6,
                name: "最近更新用户".to_string(),
                username: "recentuser".to_string(),
                email: None,
                phone: None,
                role: None,
                tenant_name: VALID_TENANT_NAME.to_string(),
                created_at: now,
                updated_at: now, // Just updated
                deleted_at: None,
            };

            assert!(dto.recently_updated());
        }

        #[test]
        fn test_constructor() {
            let created_at = Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap();
            let updated_at = Utc.with_ymd_and_hms(2024, 3, 2, 0, 0, 0).unwrap();
            
            let dto = UserResponseDto::new(
                10,
                "构造用户",
                "constructeduser",
                "构造租户",
                created_at,
                updated_at,
            );

            assert_eq!(dto.id, 10);
            assert_eq!(dto.name, "构造用户");
            assert_eq!(dto.username, "constructeduser");
            assert_eq!(dto.tenant_name, "构造租户");
            assert_eq!(dto.created_at, created_at);
            assert_eq!(dto.updated_at, updated_at);
            assert!(dto.email.is_none());
            assert!(dto.phone.is_none());
            assert!(dto.role.is_none());
            assert!(dto.deleted_at.is_none());
        }

        #[test]
        fn test_serialization_with_all_optional_fields_none() {
            let dto = UserResponseDto {
                id: 1,
                name: VALID_NAME.to_string(),
                username: VALID_USERNAME.to_string(),
                email: None,
                phone: None,
                role: None,
                tenant_name: VALID_TENANT_NAME.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };
            let json = serde_json::to_string(&dto).unwrap();
            let _deserialized: UserResponseDto = serde_json::from_str(&json).unwrap();
            assert_eq!(dto.name, VALID_NAME);
            assert_eq!(dto.username, VALID_USERNAME);
            assert!(dto.email.is_none());
            assert!(dto.role.is_none());
        }
    }

    // Tests for UserUpdateDto
    mod user_update_dto_tests {
        use super::*;

        #[test]
        fn test_valid_user_update() {
            let dto = create_valid_user_update_dto();
            assert!(dto.validate().is_ok());
            assert_eq!(dto.name, VALID_NAME);
            assert_eq!(dto.email, VALID_EMAIL);
            assert_eq!(dto.phone, VALID_PHONE);
        }

        #[test]
        fn test_serialization() {
            let dto = create_valid_user_update_dto();
            let json = serde_json::to_string(&dto).unwrap();
            let _deserialized: UserUpdateDto = serde_json::from_str(&json).unwrap();
            assert_eq!(dto.name, VALID_NAME);
            assert_eq!(dto.email, VALID_EMAIL);
            assert_eq!(dto.phone, VALID_PHONE);
        }

        #[test]
        fn test_deserialization() {
            let json_data = json!({
                "name": "更新名字",
                "email": "updated@example.com",
                "phone": "13999888777"
            });

            let dto: UserUpdateDto = serde_json::from_value(json_data).unwrap();
            assert_eq!(dto.name, "更新名字");
            assert_eq!(dto.email, "updated@example.com");
            assert_eq!(dto.phone, "13999888777");
        }

        #[test]
        fn test_name_length_validation() {
            // Valid name
            let dto = UserUpdateDto::new(MIN_NAME, VALID_EMAIL, VALID_PHONE);
            assert!(dto.validate().is_ok());

            // Too short name
            let dto = UserUpdateDto::new(TOO_SHORT_NAME, VALID_EMAIL, VALID_PHONE);
            assert_validation_error(dto.validate(), "name");

            // Too long name
            let dto = UserUpdateDto::new(TOO_LONG_NAME, VALID_EMAIL, VALID_PHONE);
            assert_validation_error(dto.validate(), "name");
        }

        #[test]
        fn test_email_validation() {
            // Invalid email
            let dto = UserUpdateDto::new(VALID_NAME, INVALID_EMAIL, VALID_PHONE);
            assert_validation_error(dto.validate(), "email");
        }

        #[test]
        fn test_phone_validation() {
            // Invalid phone
            let dto = UserUpdateDto::new(VALID_NAME, VALID_EMAIL, INVALID_PHONE);
            assert_validation_error(dto.validate(), "phone");
        }

        #[test]
        fn test_constructor() {
            let dto = UserUpdateDto::new("新名字", "new@example.com", "13666555444");
            assert_eq!(dto.name, "新名字");
            assert_eq!(dto.email, "new@example.com");
            assert_eq!(dto.phone, "13666555444");
        }

        #[test]
        fn test_business_logic_methods() {
            let dto = create_valid_user_update_dto();
            assert!(dto.has_valid_name_length());

            let invalid_dto = UserUpdateDto::new(TOO_SHORT_NAME, VALID_EMAIL, VALID_PHONE);
            assert!(!invalid_dto.has_valid_name_length());
        }

        #[test]
        fn test_clone_and_equality() {
            let dto1 = create_valid_user_update_dto();
            let dto2 = dto1.clone();
            assert_eq!(dto1, dto2);

            let dto3 = UserUpdateDto::new("不同名字", VALID_EMAIL, VALID_PHONE);
            assert_ne!(dto1, dto3);
        }
    }

    // Integration tests
    mod integration_tests {
        use super::*;

        #[test]
        fn test_user_lifecycle_workflow() {
            // Create user
            let create_dto = create_valid_user_create_dto();
            assert!(create_dto.validate().is_ok());

            // Simulate user response after creation
            let response_dto = UserResponseDto::new(
                1,
                &create_dto.name,
                &create_dto.username,
                VALID_TENANT_NAME,
                Utc::now(),
                Utc::now(),
            );
            assert_eq!(response_dto.name, create_dto.name);
            assert_eq!(response_dto.username, create_dto.username);

            // Update user
            let update_dto = UserUpdateDto::new("新名字", "new@example.com", "13999888777");
            assert!(update_dto.validate().is_ok());
        }

        #[test]
        fn test_serialization_deserialization_round_trip() {
            // Test UserCreateDto
            let create_dto = create_valid_user_create_dto();
            let json = serde_json::to_string(&create_dto).unwrap();
            let _deserialized: UserCreateDto = serde_json::from_str(&json).unwrap();
            assert_eq!(create_dto.name, "张三");
            assert_eq!(create_dto.role, "admin");

            // Test UserResponseDto
            let response_dto = create_valid_user_response_dto();
            let json = serde_json::to_string(&response_dto).unwrap();
            let _deserialized: UserResponseDto = serde_json::from_str(&json).unwrap();
            assert_eq!(response_dto.name, "张三");
            assert_eq!(response_dto.role, Some("admin".to_string()));

            // Test UserUpdateDto
            let update_dto = create_valid_user_update_dto();
            let json = serde_json::to_string(&update_dto).unwrap();
            let _deserialized: UserUpdateDto = serde_json::from_str(&json).unwrap();
            assert_eq!(update_dto.name, VALID_NAME);
            assert_eq!(update_dto.email, VALID_EMAIL);
            assert_eq!(update_dto.phone, VALID_PHONE);
        }

        #[test]
        fn test_multiple_validation_errors() {
            let dto = UserCreateDto::builder()
                .name(TOO_SHORT_NAME)
                .username(TOO_SHORT_USERNAME)
                .password(INVALID_PASSWORD)
                .email(INVALID_EMAIL)
                .phone(INVALID_PHONE)
                .role(TOO_SHORT_ROLE)
                .build();

            let result = dto.validate();
            assert!(result.is_err());
            
            let errors = result.unwrap_err();
            assert!(errors.field_errors().contains_key("name"));
            assert!(errors.field_errors().contains_key("username"));
            assert!(errors.field_errors().contains_key("password"));
            assert!(errors.field_errors().contains_key("email"));
            assert!(errors.field_errors().contains_key("phone"));
            assert!(errors.field_errors().contains_key("role"));
        }

        #[test]
        fn test_edge_cases() {
            // Test with minimum valid lengths
            let dto = UserCreateDto::builder()
                .name(MIN_NAME)
                .username(MIN_USERNAME)
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role(MIN_ROLE)
                .build();
            assert!(dto.validate().is_ok());

            // Test UserResponseDto with all optional fields None
            let dto = UserResponseDto {
                id: 1,
                name: VALID_NAME.to_string(),
                username: VALID_USERNAME.to_string(),
                email: None,
                phone: None,
                role: None,
                tenant_name: VALID_TENANT_NAME.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };
            let json = serde_json::to_string(&dto).unwrap();
            let _deserialized: UserResponseDto = serde_json::from_str(&json).unwrap();
            assert_eq!(dto.name, VALID_NAME);
            assert_eq!(dto.username, VALID_USERNAME);
            assert!(dto.email.is_none());
            assert!(dto.role.is_none());
        }

        #[test]
        fn test_special_characters_in_chinese() {
            let dto = UserCreateDto::builder()
                .name("测试用户")
                .username("testuser")
                .password(VALID_PASSWORD)
                .email(VALID_EMAIL)
                .phone(VALID_PHONE)
                .role("管理员")
                .build();
            assert!(dto.validate().is_ok());

            let json = serde_json::to_string(&dto).unwrap();
            let _deserialized: UserCreateDto = serde_json::from_str(&json).unwrap();
            assert_eq!(dto.name, "测试用户");
            assert_eq!(dto.role, "管理员");
        }
    }
}
