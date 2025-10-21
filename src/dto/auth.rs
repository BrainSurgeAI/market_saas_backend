use super::validate_phone;
use crate::utils::validator::validate_password;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Debug, Validate, PartialEq, Eq, Clone, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 4, max = 16))]
    pub username: String,

    #[validate(length(min = 8, max = 16))]
    pub password: String,
}

#[derive(Deserialize, Debug, Validate, PartialEq, Eq, Clone, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 2, max = 16))]
    pub name: String,

    #[validate(length(min = 4, max = 16))]
    pub username: String,

    #[validate(length(min = 8, max = 16))]
    pub password: String,

    #[validate(length(min = 4, max = 16))]
    pub tenant_type: String,

    #[validate(length(min = 2, max = 255))]
    pub tenant_name: String,
}

#[derive(Deserialize, Serialize, Debug, Validate, PartialEq, Eq, Clone, ToSchema)]
pub struct ResetPasswordRequest {
    #[validate(custom(function = validate_password))]
    #[serde(rename = "currentPassword")]
    pub current_password: String,

    #[validate(custom(function = validate_password))]
    #[serde(rename = "newPassword")]
    pub new_password: String,
}

/// # Super Admin Login Request
///
/// ## Fields
/// * `username` - Username of the super admin
/// * `password` - Password of the super admin
/// * `tenant_type` - Type of the tenant
#[derive(Deserialize, Debug, Validate, PartialEq, Eq, Clone, ToSchema)]
pub struct SuperAdminLoginRequest {
    #[validate(length(min = 4, max = 16))]
    pub username: String,

    #[validate(length(min = 8, max = 16))]
    pub password: String,

    #[validate(custom(function = validate_phone))]
    #[serde(rename = "phone")]
    pub phone_number: String,

    #[validate(length(min = 6, max = 6))]
    #[serde(rename = "verificationCode")]
    pub code: String,
}

/// # Verify Code Request
///
/// ## Fields
/// * `phone_number` - Phone number of the user
#[derive(Deserialize, Debug, Validate, PartialEq, Eq, Clone, ToSchema)]
pub struct VerifyCodeRequestDTO {
    #[validate(custom(function = validate_phone))]
    #[serde(rename = "phone")]
    pub phone_number: String,
}

/// # Verify Code Response
///
/// ## Fields
/// * `code` - Code of the user
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct VerifyCodeResponseDTO {
    pub code: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use validator::Validate;

    // Test constants for better maintainability
    const VALID_USERNAME_MIN: &str = "test";  // 4 chars
    const VALID_USERNAME_MAX: &str = "1234567890123456";  // 16 chars
    const VALID_PASSWORD_MIN: &str = "Pass123!";  // 8 chars
    const VALID_PASSWORD_MAX: &str = "1234567890Abc!@#";  // 16 chars
    const VALID_PHONE: &str = "13800138000";
    const VALID_CODE: &str = "123456";
    const INVALID_PHONE: &str = "1234567890";

    // Helper functions to reduce code duplication
    fn create_login_request(username: &str, password: &str) -> LoginRequest {
        let json_data = json!({
            "username": username,
            "password": password
        });
        serde_json::from_value(json_data).unwrap()
    }

    fn create_register_request(name: &str, username: &str, password: &str, tenant_type: &str, tenant_name: &str) -> RegisterRequest {
        let json_data = json!({
            "name": name,
            "username": username,
            "password": password,
            "tenant_type": tenant_type,
            "tenant_name": tenant_name
        });
        serde_json::from_value(json_data).unwrap()
    }

    fn create_reset_password_request(current_password: &str, new_password: &str) -> ResetPasswordRequest {
        let json_data = json!({
            "currentPassword": current_password,
            "newPassword": new_password
        });
        serde_json::from_value(json_data).unwrap()
    }

    fn create_super_admin_login_request(username: &str, password: &str, phone: &str, code: &str) -> SuperAdminLoginRequest {
        let json_data = json!({
            "username": username,
            "password": password,
            "phone": phone,
            "verificationCode": code
        });
        serde_json::from_value(json_data).unwrap()
    }

    fn create_verify_code_request(phone: &str) -> VerifyCodeRequestDTO {
        let json_data = json!({
            "phone": phone
        });
        serde_json::from_value(json_data).unwrap()
    }

    // Tests for LoginRequest
    mod login_request_tests {
        use super::*;

        #[test]
        fn test_valid_login() {
            let login_req = create_login_request("testuser", "password123");
            assert!(login_req.validate().is_ok());
            assert_eq!(login_req.username, "testuser");
            assert_eq!(login_req.password, "password123");
        }

        #[test]
        fn test_boundary_values() {
            // Test minimum length boundaries
            let login_req = create_login_request(VALID_USERNAME_MIN, VALID_PASSWORD_MIN);
            assert!(login_req.validate().is_ok());

            // Test maximum length boundaries
            let login_req = create_login_request(VALID_USERNAME_MAX, VALID_PASSWORD_MAX);
            assert!(login_req.validate().is_ok());
        }

        #[test]
        fn test_username_validation_failures() {
            // Username too short
            let login_req = create_login_request("ab", "password123");
            assert!(login_req.validate().is_err());

            // Username too long
            let login_req = create_login_request("verylongusernamethatexceedslimit", "password123");
            assert!(login_req.validate().is_err());

            // Username empty
            let login_req = create_login_request("", "password123");
            assert!(login_req.validate().is_err());
        }

        #[test]
        fn test_password_validation_failures() {
            // Password too short
            let login_req = create_login_request("testuser", "short");
            assert!(login_req.validate().is_err());

            // Password too long
            let login_req = create_login_request("testuser", "verylongpasswordthatexceedslimit");
            assert!(login_req.validate().is_err());

            // Password empty
            let login_req = create_login_request("testuser", "");
            assert!(login_req.validate().is_err());
        }
    }

    // Tests for RegisterRequest
    mod register_request_tests {
        use super::*;

        #[test]
        fn test_valid_registration() {
            let register_req = create_register_request("John Doe", "testuser", "password123", "business", "Test Company");
            assert!(register_req.validate().is_ok());
        }

        #[test]
        fn test_field_validation_failures() {
            // Name too short
            let register_req = create_register_request("a", "testuser", "password123", "business", "Test Company");
            assert!(register_req.validate().is_err());

            // Tenant type too short
            let register_req = create_register_request("John Doe", "testuser", "password123", "bu", "Test Company");
            assert!(register_req.validate().is_err());

            // Tenant name too short
            let register_req = create_register_request("John Doe", "testuser", "password123", "business", "a");
            assert!(register_req.validate().is_err());
        }

        #[test]
        fn test_boundary_values() {
            // Test minimum values
            let register_req = create_register_request("Jo", "test", "12345678", "busi", "Co");
            assert!(register_req.validate().is_ok());

            // Test maximum values
            let register_req = create_register_request(
                "1234567890123456",  // 16 chars for name
                "1234567890123456",  // 16 chars for username
                "1234567890123456",  // 16 chars for password
                "1234567890123456",  // 16 chars for tenant_type
                &"a".repeat(255)     // 255 chars for tenant_name
            );
            assert!(register_req.validate().is_ok());
        }
    }

    // Tests for ResetPasswordRequest
    mod reset_password_request_tests {
        use super::*;

        #[test]
        fn test_valid_password_reset() {
            let reset_req = create_reset_password_request("OldPass123!", "NewPass456@");
            assert!(reset_req.validate().is_ok());
        }

        #[test]
        fn test_invalid_password_formats() {
            let test_cases = vec![
                ("oldpassword123", "NewPass456@", "Missing uppercase and special chars"),
                ("OLDPASSWORD123", "NewPass456@", "Missing lowercase and special chars"),
                ("OldPassword", "NewPass456@", "Missing digits and special chars"),
                ("OldPass123", "NewPass456@", "Missing special chars"),
                ("OldPass!", "NewPass456@", "Missing digits"),
                ("Old Pass123!", "NewPass456@", "Contains space"),
                ("Short1!", "NewPass456@", "Too short"),
            ];

            for (current_pass, new_pass, description) in test_cases {
                let reset_req = create_reset_password_request(current_pass, new_pass);
                assert!(reset_req.validate().is_err(), "Should fail for: {}", description);
            }
        }

        #[test]
        fn test_password_length_boundaries() {
            // Test exact minimum length
            let reset_req = create_reset_password_request(VALID_PASSWORD_MIN, "NewPass456@");
            assert!(reset_req.validate().is_ok());

            // Test exact maximum length
            let reset_req = create_reset_password_request("OldPass123!", VALID_PASSWORD_MAX);
            assert!(reset_req.validate().is_ok());
        }

        #[test]
        fn test_serialization_field_names() {
            let reset_req = ResetPasswordRequest {
                current_password: "OldPass123!".to_string(),
                new_password: "NewPass456@".to_string(),
            };
            
            let json_value = serde_json::to_value(&reset_req).unwrap();
            assert!(json_value.get("currentPassword").is_some());
            assert!(json_value.get("newPassword").is_some());
            assert!(json_value.get("current_password").is_none());
            assert!(json_value.get("new_password").is_none());
        }
    }

    // Tests for SuperAdminLoginRequest
    mod super_admin_login_tests {
        use super::*;

        #[test]
        fn test_valid_super_admin_login() {
            let super_admin_req = create_super_admin_login_request("admin", "AdminPass123!", VALID_PHONE, VALID_CODE);
            assert!(super_admin_req.validate().is_ok());
        }

        #[test]
        fn test_invalid_phone_numbers() {
            let invalid_phones = vec![
                "1234567890",    // Too short
                "123456789012",  // Too long
                "23456789012",   // Doesn't start with 1
                "1abcdefghij",   // Contains letters
                "",              // Empty
            ];

            for phone in invalid_phones {
                let super_admin_req = create_super_admin_login_request("admin", "AdminPass123!", phone, VALID_CODE);
                assert!(super_admin_req.validate().is_err(), "Should fail for phone: {}", phone);
            }
        }

        #[test]
        fn test_invalid_verification_codes() {
            let invalid_codes = vec![
                "12345",    // Too short
                "1234567",  // Too long
                "",         // Empty
            ];

            for code in invalid_codes {
                let super_admin_req = create_super_admin_login_request("admin", "AdminPass123!", VALID_PHONE, code);
                assert!(super_admin_req.validate().is_err(), "Should fail for code: {}", code);
            }
        }
    }

    // Tests for VerifyCodeRequestDTO
    mod verify_code_tests {
        use super::*;

        #[test]
        fn test_valid_verify_code_request() {
            let verify_req = create_verify_code_request(VALID_PHONE);
            assert!(verify_req.validate().is_ok());
        }

        #[test]
        fn test_invalid_phone_numbers() {
            let verify_req = create_verify_code_request(INVALID_PHONE);
            assert!(verify_req.validate().is_err());
        }
    }

    // Tests for VerifyCodeResponseDTO (no validation, just deserialization)
    mod verify_code_response_tests {
        use super::*;

        #[test]
        fn test_deserialization() {
            let json_data = json!({
                "code": "123456"
            });
            
            let verify_resp: VerifyCodeResponseDTO = serde_json::from_value(json_data).unwrap();
            assert_eq!(verify_resp.code, "123456");
        }
    }

    // General struct behavior tests
    mod struct_behavior_tests {
        use super::*;

        #[test]
        fn test_struct_equality() {
            let login1 = LoginRequest {
                username: "test".to_string(),
                password: "password123".to_string(),
            };
            
            let login2 = LoginRequest {
                username: "test".to_string(),
                password: "password123".to_string(),
            };
            
            let login3 = LoginRequest {
                username: "different".to_string(),
                password: "password123".to_string(),
            };
            
            assert_eq!(login1, login2);
            assert_ne!(login1, login3);
        }

        #[test]
        fn test_debug_output() {
            let login_req = LoginRequest {
                username: "test".to_string(),
                password: "password123".to_string(),
            };
            
            let debug_output = format!("{:?}", login_req);
            assert!(debug_output.contains("LoginRequest"));
            assert!(debug_output.contains("test"));
            assert!(debug_output.contains("password123"));
        }

        #[test]
        fn test_clone_and_partial_eq() {
            let login_req = LoginRequest {
                username: "test".to_string(),
                password: "password123".to_string(),
            };
            
            let cloned_req = login_req.clone();
            assert_eq!(login_req, cloned_req);
            
            // Test ResetPasswordRequest clone as well
            let reset_req = ResetPasswordRequest {
                current_password: "OldPass123!".to_string(),
                new_password: "NewPass456@".to_string(),
            };
            
            let cloned_reset_req = reset_req.clone();
            assert_eq!(reset_req, cloned_reset_req);
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_clone_performance_impact() {
        println!("\n=== Clone Performance Analysis ===");
        
        // Create test data
        let login_req = LoginRequest {
            username: "a".repeat(16),    // Max length username
            password: "a".repeat(16),    // Max length password
        };
        
        let reset_req = ResetPasswordRequest {
            current_password: "Pass123!@#$%^&*()".to_string(),
            new_password: "NewPass456@#$%^&*()".to_string(),
        };
        
        // Test clone performance
        let iterations = 100_000;
        
        // Test LoginRequest clone
        let start = Instant::now();
        for _ in 0..iterations {
            let _cloned = login_req.clone();
        }
        let login_duration = start.elapsed();
        
        // Test ResetPasswordRequest clone
        let start = Instant::now();
        for _ in 0..iterations {
            let _cloned = reset_req.clone();
        }
        let reset_duration = start.elapsed();
        
        // Test memory usage estimation
        let login_size = std::mem::size_of::<LoginRequest>();
        let reset_size = std::mem::size_of::<ResetPasswordRequest>();
        
        println!("LoginRequest size: {} bytes", login_size);
        println!("ResetPasswordRequest size: {} bytes", reset_size);
        println!("LoginRequest clone time: {:?} for {} iterations", login_duration, iterations);
        println!("ResetPasswordRequest clone time: {:?} for {} iterations", reset_duration, iterations);
        println!("Average clone time per LoginRequest: {:?}", login_duration / iterations);
        println!("Average clone time per ResetPasswordRequest: {:?}", reset_duration / iterations);
        
        // Performance assertions (these are reasonable thresholds)
        assert!(login_duration.as_millis() < 100, "Clone operation too slow");
        assert!(reset_duration.as_millis() < 100, "Clone operation too slow");
    }
    
    #[test]
    fn test_memory_allocation_comparison() {
        println!("\n=== Memory Allocation Analysis ===");
        
        // Test without clone (reference)
        let login_req = LoginRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };
        
        // Simulate passing by reference (no allocation)
        fn process_by_ref(req: &LoginRequest) -> bool {
            req.username.len() > 0 && req.password.len() > 0
        }
        
        // Simulate passing by clone (allocation)
        fn process_by_clone(req: LoginRequest) -> bool {
            req.username.len() > 0 && req.password.len() > 0
        }
        
        let iterations = 10_000;
        
        // Test by reference
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = process_by_ref(&login_req);
        }
        let ref_duration = start.elapsed();
        
        // Test by clone
        let start = Instant::now();
        for _ in 0..iterations {
            let _result = process_by_clone(login_req.clone());
        }
        let clone_duration = start.elapsed();
        
        println!("Processing by reference: {:?} for {} iterations", ref_duration, iterations);
        println!("Processing by clone: {:?} for {} iterations", clone_duration, iterations);
        println!("Clone overhead: {}x slower", clone_duration.as_nanos() as f64 / ref_duration.as_nanos() as f64);
        
        // In most cases, reference should be much faster
        assert!(ref_duration < clone_duration, "Reference should be faster than clone");
    }
}
