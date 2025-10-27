//use anyhow::Ok;
use regex::Regex;
use validator::ValidationError;

/// Validates a username string against the following requirements:
/// - Must be between 6 and 32 characters long
/// - Can only contain alphanumeric characters (a-z, A-Z, 0-9)
/// - No spaces or special characters allowed
///
/// # Arguments
/// * `username` - The username string to validate
///
/// # Returns
/// * `bool` - True if username meets all requirements, false otherwise
///

pub(crate) fn validate_username(username: &str) -> bool {
    let username_regex = Regex::new(r"^[a-zA-Z0-9]{6,32}$").unwrap();
    username_regex.is_match(username)
}

/// Validates a password string against the following requirements:
/// - At least 8 characters long
/// - At most 16 characters long
/// - Contains at least one uppercase letter
/// - Contains at least one lowercase letter
/// - Contains at least one digit
/// - Contains at least one special character (!@#$%^&*()_+-=[]{}\\|;:'\",.<>/?)
/// - Does not contain spaces
///
/// # Arguments
/// * `password` - The password string to validate
///
/// # Returns
/// * `bool` - True if password meets all requirements, false otherwise
///

pub(crate) fn check_password(password: &str) -> bool {
    if password.len() < 8 || password.len() > 16 || password.contains(' ') {
        return false;
    }

    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}\\|;:'\",.<>/?".contains(c));

    has_upper && has_lower && has_digit && has_special
}

// 适配函数，将bool返回转换为Result返回
pub(crate) fn validate_password(password: &str) -> Result<(), ValidationError> {
    if check_password(password) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid password format"))
    }
}

pub(crate) fn validate_email(email: &str) -> Result<(), ValidationError> {
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$"
    ).unwrap();

    if !email_regex.is_match(email) || email.contains("..") {
        return Err(ValidationError::new("Invalid email address"));
    }

    if email.len() > 255 {
        return Err(ValidationError::new("Email address is too long"));
    }
    Ok(())
}

pub(crate) fn validate_apprive_status(status: &str) -> Result<(), ValidationError> {
    if status.to_lowercase() != "approved" && status.to_lowercase() != "rejected" {
        return Err(ValidationError::new("Invalid status"));
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        // Valid usernames
        assert!(validate_username("user123"));
        assert!(validate_username("ADMIN456"));
        assert!(validate_username("JohnDoe789"));
        assert!(validate_username("abcdef")); // Minimum length
        assert!(validate_username("a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6")); // Maximum length

        // Invalid usernames
        assert!(!validate_username("user")); // Too short
        assert!(!validate_username("user name")); // Contains space
        assert!(!validate_username("user@123")); // Contains special character
        assert!(!validate_username("user-123")); // Contains hyphen
        assert!(!validate_username("user_123")); // Contains underscore
        assert!(!validate_username("")); // Empty string
        assert!(!validate_username("a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7")); // Too long
    }

    #[test]
    fn test_validate_password() {
        // Valid passwords
        assert!(check_password("Pass1234!"));
        assert!(check_password("Complex@123"));
        assert!(check_password("Abcd123$xyz"));
        assert!(check_password("P@ssw0rd"));
        assert!(check_password("Test123!@#"));

        // Invalid passwords
        assert!(!check_password("pass")); // Too short
        assert!(!check_password("password123")); // No uppercase, no special char
        assert!(!check_password("PASSWORD123")); // No lowercase, no special char
        assert!(!check_password("Password123")); // No special char
        assert!(!check_password("Pass word!")); // Contains space
        assert!(!check_password("abcdefghijklmnop1!")); // Too long
        assert!(!check_password("")); // Empty string
        assert!(!check_password("Pass1234")); // No special char
        assert!(!check_password("PASS!@#$")); // No lowercase, no number
        assert!(!check_password("pass!@#$")); // No uppercase, no number
    }

    #[test]
    fn test_validate_email() {
        // Valid email addresses
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("user.name@example.com").is_ok());
        assert!(validate_email("user+tag@example.com").is_ok());
        assert!(validate_email("user123@example.co.uk").is_ok());
        assert!(validate_email("first.last@subdomain.example.com").is_ok());

        // Invalid email addresses
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid.email").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("user@.com").is_err());
        assert!(validate_email("user@example").is_err());
        assert!(validate_email("user name@example.com").is_err());
        assert!(validate_email("user@exam ple.com").is_err());
        assert!(validate_email("user@@example.com").is_err());
        assert!(validate_email("user@example..com").is_err());
    }

    #[test]
    fn test_validate_approve_status() {
        assert!(validate_apprive_status("PUBLISHED").is_err());
        assert!(validate_apprive_status("Unknown").is_err());
        assert!(validate_apprive_status("REJECT").is_ok());
        assert!(validate_apprive_status("APPROVE").is_ok());
        assert!(validate_apprive_status("reject").is_ok());
        assert!(validate_apprive_status("approve").is_ok());
    }
}
