use anyhow::Result;
use serde_json::json;
use std::sync::LazyLock;

use crate::{
    libs::permission::{self, PERMISSION_TRIES},
    models::claims::Claims,
};
use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use hyper::{header, Method, StatusCode};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use tracing::{debug, error, info, warn};

// Performance monitoring threshold (milliseconds)
const AUTH_SLOW_THRESHOLD_MS: u64 = 100;

// JWT validation configuration
const JWT_ALGORITHM: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS512;

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

// Pre-configured JWT validation settings to avoid creating them every time
static JWT_VALIDATION: LazyLock<Validation> = LazyLock::new(|| {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.validate_exp = true;
    validation
});

/// Creates a JWT token for a user with the specified roles and permissions
///
/// # Arguments
/// * `claims` - Claims to include in the token
///
/// # Returns   
/// * `Ok(String)` - JWT token string on successful creation
/// * `Err(AuthError)` - TokenCreation error if token creation fails
pub fn create_jwt(claims: &Claims) -> Result<String, AuthError> {
    let start_time = std::time::Instant::now();
    let header = Header::new(JWT_ALGORITHM);

    let result = encode(&header, &claims, &KEYS.encoding).map_err(|e| {
        error!(
            "Failed to create JWT token for user {}: {}",
            claims.username, e
        );
        AuthError::TokenCreation
    });

    let duration = start_time.elapsed();
    if duration.as_millis() > AUTH_SLOW_THRESHOLD_MS as u128 {
        warn!(
            "Slow JWT creation for user {}: {:?}",
            claims.username, duration
        );
    }

    result
}

/// Decodes a JWT token and returns the claims
///
/// # Arguments
/// * `token` - JWT token string to decode
///
/// # Returns
/// * `Ok(Claims)` - Claims object on successful decoding
/// * `Err(AuthError)` - InvalidToken error if token decoding fails
fn decode_jwt(token: &str) -> Result<Claims, AuthError> {
    let start_time = std::time::Instant::now();

    let claims = decode::<Claims>(token, &KEYS.decoding, &JWT_VALIDATION).map_err(|e| {
        warn!("Failed to decode JWT token: {}", e);
        AuthError::InvalidToken
    })?;

    let duration = start_time.elapsed();
    if duration.as_millis() > AUTH_SLOW_THRESHOLD_MS as u128 {
        warn!("Slow JWT decoding: {:?}", duration);
    }

    debug!(
        "Successfully decoded JWT for user: {}",
        claims.claims.username
    );
    Ok(claims.claims)
}

/// Middleware function to handle authentication and JWT token validation
///
/// # Arguments
/// * `req` - The incoming HTTP request
/// * `next` - The next middleware/handler in the chain
///
/// # Returns
/// * `Result<Response, StatusCode>` - Success response or appropriate error status code
///
/// This middleware:
/// 1. Extracts and validates the Authorization header
/// 2. Decodes and validates the JWT token
/// 3. Performs access control verification
/// 4. Injects the claims into request extensions for downstream handlers
pub async fn auth_middleware(mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    // 1. Extract and validate Authorization header
    let auth_header = req.headers().get(header::AUTHORIZATION).ok_or_else(|| {
        warn!("Missing Authorization header for {} {}", method, path);
        StatusCode::UNAUTHORIZED
    })?;

    let auth_str = auth_header.to_str().map_err(|_| {
        warn!(
            "Invalid Authorization header format for {} {}",
            method, path
        );
        StatusCode::BAD_REQUEST
    })?;

    let token = auth_str.strip_prefix("Bearer ").ok_or_else(|| {
        warn!(
            "Authorization header missing 'Bearer ' prefix for {} {}",
            method, path
        );
        StatusCode::BAD_REQUEST
    })?;

    // 2. Decode and validate JWT token
    let claims = decode_jwt(token).map_err(|_| {
        warn!("Invalid JWT token for {} {}", method, path);
        StatusCode::UNAUTHORIZED
    })?;

    // 3. Verify access permissions
    verify_access(&path, &method, &claims)
        .await
        .inspect_err(|&status| {
            warn!(
                "Access denied for user {} to {} {}: {:?}",
                claims.username, method, path, status
            );
        })?;

    // 4. Inject claims into request extensions
    req.extensions_mut().insert(claims.clone());

    let auth_duration = start_time.elapsed();
    if auth_duration.as_millis() > AUTH_SLOW_THRESHOLD_MS as u128 {
        warn!(
            "Slow auth middleware for user {} on {} {}: {:?}",
            claims.username, method, path, auth_duration
        );
    } else {
        debug!(
            "Auth middleware completed for user {} on {} {} in {:?}",
            claims.username, method, path, auth_duration
        );
    }

    Ok(next.run(req).await)
}

/// Verifies if the user has permission to access the requested resource
///
/// # Arguments
/// * `path` - The request path to verify
/// * `method` - The HTTP method being used
/// * `claims` - The JWT claims containing user information and permissions
///
/// # Returns
/// * `Result<(), StatusCode>` - Ok if access is granted, error status code otherwise
///
/// This function performs several access control checks:
/// 1. Super admin bypass check
/// 2. Route pattern matching against permission tries
/// 3. Self-only route validation (username matching)
/// 4. Tenant-specific route validation (tenant_hash matching)
/// 5. Permission requirement validation
async fn verify_access(path: &str, method: &Method, claims: &Claims) -> Result<(), StatusCode> {
    debug!(
        "Verifying access for user {} to {} {}",
        claims.username, method, path
    );

    // Super admin bypass - early return for performance
    if claims.is_super_admin {
        debug!("Super admin access granted for user {}", claims.username);
        return Ok(());
    }

    let tries = PERMISSION_TRIES.read().await;

    let trie = tries.get(method).ok_or_else(|| {
        error!("No permission trie found for HTTP method: {}", method);
        StatusCode::METHOD_NOT_ALLOWED
    })?;

    let (rule, params) = trie.find(path).ok_or_else(|| {
        error!("No matching route pattern found for {} {}", method, path);
        StatusCode::NOT_FOUND
    })?;

    debug!("Matched rule: {:?} for path: {}", rule, path);
    debug!("Captured params: {:?}", params);

    // Validate self-only routes
    if rule.self_only {
        if let Some(username) = params.get("username") {
            if username != &claims.username {
                error!(
                    "Self-only route {} accessed by user {} (expected: {})",
                    path, claims.username, username
                );
                return Err(StatusCode::FORBIDDEN);
            }
        }
        // TODO: Check if provider_hash in token matches provider_hash in path
    }

    // Validate tenant access
    if let Some(tenant_hash) = params.get("tenant_hash") {
        let has_tenant_access = claims.tenant_hash == *tenant_hash;
        let is_market_admin = claims
            .roles
            .first()
            .map(|role| role.to_uppercase() == "MARKET_ADMIN")
            .unwrap_or(false);

        if !has_tenant_access && !is_market_admin {
            error!(
                "User {} (tenant: {}) does not have access to tenant: {}",
                claims.username, claims.tenant_hash, tenant_hash
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Validate permissions
    if !permission::verify_permissions(&rule.required_permission, &claims.permissions) {
        error!(
            "User {} does not have required permission '{}'. User permissions: {:?}",
            claims.username, rule.required_permission, claims.permissions
        );
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        "Access granted to user {} for {} {} (permission: {})",
        claims.username, method, path, rule.required_permission
    );
    Ok(())
}

struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AuthError {
    TokenCreation,
    InvalidToken,
    ExpiredToken,
    MissingToken,
    InvalidFormat,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::TokenCreation => write!(f, "Failed to create JWT token"),
            AuthError::InvalidToken => write!(f, "Invalid JWT token"),
            AuthError::ExpiredToken => write!(f, "JWT token has expired"),
            AuthError::MissingToken => write!(f, "Missing authentication token"),
            AuthError::InvalidFormat => write!(f, "Invalid token format"),
        }
    }
}

impl std::error::Error for AuthError {}

impl IntoResponse for AuthError {
    /// Converts an AuthError into an HTTP response with appropriate status code and message
    fn into_response(self) -> Response {
        let (status, error_code, error_message) = match self {
            AuthError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "TOKEN_CREATION_ERROR",
                "Failed to create authentication token",
            ),
            AuthError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "The provided token is invalid",
            ),
            AuthError::ExpiredToken => (
                StatusCode::UNAUTHORIZED,
                "EXPIRED_TOKEN",
                "The authentication token has expired",
            ),
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "MISSING_TOKEN",
                "Authentication token is required",
            ),
            AuthError::InvalidFormat => (
                StatusCode::BAD_REQUEST,
                "INVALID_TOKEN_FORMAT",
                "Invalid authentication token format",
            ),
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": error_message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use hyper::Method;

    // Test Claims data
    fn create_test_claims() -> Claims {
        Claims {
            tenant_type: "PROVIDER".to_string(),
            tenant_name: "Test Provider".to_string(),
            tenant_hash: "test_provider_hash".to_string(),
            username: "test_user".to_string(),
            roles: vec!["PROVIDER".to_string()],
            permissions: vec!["order:read".to_string(), "order:write".to_string()],
            exp: (Utc::now().timestamp() + 3600) as usize, // Expires in 1 hour
            is_super_admin: false,
        }
    }

    // Helper function to setup test environment with clean state
    async fn setup_test_environment() {
        // Clear any existing permission tries
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();
        }
        // Small delay to ensure state is clean
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }

    fn create_super_admin_claims() -> Claims {
        Claims {
            tenant_type: "SUPER_ADMIN".to_string(),
            tenant_name: "SUPER_ADMIN".to_string(),
            tenant_hash: "SUPER_ADMIN".to_string(),
            username: "super_admin".to_string(),
            roles: vec!["SUPER_ADMIN".to_string()],
            permissions: vec!["*".to_string()],
            exp: (Utc::now().timestamp() + 3600) as usize,
            is_super_admin: true,
        }
    }

    fn create_expired_claims() -> Claims {
        Claims {
            tenant_type: "PROVIDER".to_string(),
            tenant_name: "Test Provider".to_string(),
            tenant_hash: "test_provider_hash".to_string(),
            username: "test_user".to_string(),
            roles: vec!["PROVIDER".to_string()],
            permissions: vec!["order:read".to_string()],
            exp: (Utc::now().timestamp() - 3600) as usize, // Expired 1 hour ago
            is_super_admin: false,
        }
    }

    // Create test Keys to avoid using global KEYS
    fn create_test_keys(secret: &str) -> Keys {
        Keys::new(secret.as_bytes())
    }

    // Create JWT with specified key
    fn create_jwt_with_key(claims: &Claims, keys: &Keys) -> Result<String, AuthError> {
        let header = Header::new(jsonwebtoken::Algorithm::HS512);
        encode(&header, &claims, &keys.encoding).map_err(|e| {
            error!("{}", e);
            AuthError::TokenCreation
        })
    }

    // Decode JWT with specified key
    fn decode_jwt_with_key(token: &str, keys: &Keys) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS512);
        validation.validate_exp = true;

        let claims = decode::<Claims>(token, &keys.decoding, &validation).map_err(|e| {
            error!("decode_jwt: {}", e);
            AuthError::InvalidToken
        })?;
        Ok(claims.claims)
    }

    #[tokio::test]
    async fn test_create_jwt_success() {
        // Set environment variable
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        let claims = create_test_claims();
        let result = create_jwt(&claims);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_decode_jwt_success() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        let claims = create_test_claims();
        let token = create_jwt(&claims).unwrap();

        let decoded_claims = decode_jwt(&token);
        assert!(decoded_claims.is_ok());

        let decoded = decoded_claims.unwrap();
        assert_eq!(decoded.username, claims.username);
        assert_eq!(decoded.tenant_hash, claims.tenant_hash);
        assert_eq!(decoded.permissions, claims.permissions);
    }

    #[tokio::test]
    async fn test_decode_jwt_invalid_token() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        let result = decode_jwt("invalid_token");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
    }

    #[tokio::test]
    async fn test_decode_jwt_expired_token() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        let expired_claims = create_expired_claims();
        let token = create_jwt(&expired_claims).unwrap();

        let result = decode_jwt(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
    }

    #[tokio::test]
    async fn test_verify_access_super_admin_bypass() {
        let claims = create_super_admin_claims();
        let path = "/api/any/path";
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_no_matching_route() {
        // Clear permission tries to simulate no matching routes
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();
        }

        let claims = create_test_claims();
        let path = "/api/nonexistent/path";
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_err());
        // Now returns METHOD_NOT_ALLOWED when no permission trie is found for HTTP method
        assert_eq!(result.unwrap_err(), StatusCode::METHOD_NOT_ALLOWED);
    }

    // Additional comprehensive tests for verify_access method

    #[tokio::test]
    async fn test_verify_access_self_only_route_success() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie with self-only route
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(), // Use permission that test user has
                self_only: true,
                path_pattern: "/api/users/{username}/profile".to_string(),
            };
            trie.insert("/api/users/{username}/profile", rule);
            tries.insert(Method::GET, trie);
        }

        let claims = create_test_claims();
        let path = "/api/users/test_user/profile"; // Matches claims.username
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_self_only_route_forbidden() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie with self-only route
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(), // Use permission that test user has
                self_only: true,
                path_pattern: "/api/users/{username}/profile".to_string(),
            };
            trie.insert("/api/users/{username}/profile", rule);
            tries.insert(Method::GET, trie);
        }

        let claims = create_test_claims();
        let path = "/api/users/other_user/profile"; // Different from claims.username
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_verify_access_tenant_access_success() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie with tenant-specific route
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: false,
                path_pattern: "/api/tenants/{tenant_hash}/orders".to_string(),
            };
            trie.insert("/api/tenants/{tenant_hash}/orders", rule);
            tries.insert(Method::GET, trie);
        }

        let claims = create_test_claims();
        let path = "/api/tenants/test_provider_hash/orders"; // Matches claims.tenant_hash
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_tenant_access_forbidden() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie with tenant-specific route
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: false,
                path_pattern: "/api/tenants/{tenant_hash}/orders".to_string(),
            };
            trie.insert("/api/tenants/{tenant_hash}/orders", rule);
            tries.insert(Method::GET, trie);
        }

        let claims = create_test_claims();
        let path = "/api/tenants/different_tenant_hash/orders"; // Different from claims.tenant_hash
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_verify_access_market_admin_bypass_tenant_restriction() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie with tenant-specific route
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: false,
                path_pattern: "/api/tenants/{tenant_hash}/orders".to_string(),
            };
            trie.insert("/api/tenants/{tenant_hash}/orders", rule);
            tries.insert(Method::GET, trie);
        }

        // Create market admin claims
        let mut claims = create_test_claims();
        claims.roles = vec!["MARKET_ADMIN".to_string()];
        claims.permissions = vec!["order:read".to_string()];

        let path = "/api/tenants/different_tenant_hash/orders"; // Different from claims.tenant_hash
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_ok()); // Market admin should bypass tenant restriction
    }

    #[tokio::test]
    async fn test_verify_access_route_not_found() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie with limited routes
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: false,
                path_pattern: "/api/orders".to_string(),
            };
            trie.insert("/api/orders", rule);
            tries.insert(Method::GET, trie);
        }

        let claims = create_test_claims();
        let path = "/api/nonexistent"; // Route not in trie
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_verify_access_case_insensitive_market_admin() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: false,
                path_pattern: "/api/tenants/{tenant_hash}/orders".to_string(),
            };
            trie.insert("/api/tenants/{tenant_hash}/orders", rule);
            tries.insert(Method::GET, trie);
        }

        // Test with lowercase market_admin role
        let mut claims = create_test_claims();
        claims.roles = vec!["market_admin".to_string()]; // lowercase
        claims.permissions = vec!["order:read".to_string()];

        let path = "/api/tenants/different_tenant_hash/orders";
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_ok()); // Should work due to to_uppercase() conversion
    }

    #[tokio::test]
    async fn test_verify_access_empty_roles() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: false,
                path_pattern: "/api/tenants/{tenant_hash}/orders".to_string(),
            };
            trie.insert("/api/tenants/{tenant_hash}/orders", rule);
            tries.insert(Method::GET, trie);
        }

        let mut claims = create_test_claims();
        claims.roles = vec![]; // Empty roles
        claims.permissions = vec!["order:read".to_string()];

        let path = "/api/tenants/different_tenant_hash/orders";
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    // Additional tests for other JWT and auth functionality

    #[test]
    fn test_auth_error_into_response() {
        let token_creation_error = AuthError::TokenCreation;
        let response = token_creation_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let invalid_token_error = AuthError::InvalidToken;
        let response = invalid_token_error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_keys_creation() {
        let secret = b"test_secret";
        let _keys = Keys::new(secret);

        // Verify Keys struct can be created correctly
        // This mainly tests that it doesn't panic
        assert!(true);
    }

    // Edge case tests
    #[tokio::test]
    async fn test_create_jwt_with_empty_permissions() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        let mut claims = create_test_claims();
        claims.permissions = vec![];

        let result = create_jwt(&claims);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_jwt_with_long_username() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        let mut claims = create_test_claims();
        claims.username = "a".repeat(100); // Very long username

        let result = create_jwt(&claims);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_decode_jwt_with_wrong_secret() {
        // Use different keys to create and decode token to test key mismatch
        let claims = create_test_claims();

        // Create token with first key
        let keys1 = create_test_keys("secret1");
        let token = create_jwt_with_key(&claims, &keys1).unwrap();

        // Try to decode with second key
        let keys2 = create_test_keys("secret2");
        let result = decode_jwt_with_key(&token, &keys2);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
    }

    // Test Claims struct fields
    #[test]
    fn test_claims_creation_and_fields() {
        let claims = create_test_claims();

        assert_eq!(claims.tenant_type, "PROVIDER");
        assert_eq!(claims.tenant_name, "Test Provider");
        assert_eq!(claims.tenant_hash, "test_provider_hash");
        assert_eq!(claims.username, "test_user");
        assert_eq!(claims.roles, vec!["PROVIDER"]);
        assert_eq!(claims.permissions, vec!["order:read", "order:write"]);
        assert!(!claims.is_super_admin);
        assert!(claims.exp > 0);
    }

    // Test super admin Claims
    #[test]
    fn test_super_admin_claims() {
        let claims = create_super_admin_claims();

        assert_eq!(claims.tenant_type, "SUPER_ADMIN");
        assert!(claims.is_super_admin);
        assert_eq!(claims.permissions, vec!["*"]);
    }

    // Test JWT token roundtrip conversion
    #[tokio::test]
    async fn test_jwt_roundtrip() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        let original_claims = create_test_claims();
        let token = create_jwt(&original_claims).unwrap();
        let decoded_claims = decode_jwt(&token).unwrap();

        // Verify all fields are correct
        assert_eq!(decoded_claims.tenant_type, original_claims.tenant_type);
        assert_eq!(decoded_claims.tenant_name, original_claims.tenant_name);
        assert_eq!(decoded_claims.tenant_hash, original_claims.tenant_hash);
        assert_eq!(decoded_claims.username, original_claims.username);
        assert_eq!(decoded_claims.roles, original_claims.roles);
        assert_eq!(decoded_claims.permissions, original_claims.permissions);
        assert_eq!(
            decoded_claims.is_super_admin,
            original_claims.is_super_admin
        );
        assert_eq!(decoded_claims.exp, original_claims.exp);
    }

    // Test different types of Claims
    #[tokio::test]
    async fn test_different_tenant_types() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        let tenant_types = vec!["PROVIDER", "CUSTOMER", "MARKET", "SUPER_ADMIN"];

        for tenant_type in tenant_types {
            let mut claims = create_test_claims();
            claims.tenant_type = tenant_type.to_string();

            let token = create_jwt(&claims).unwrap();
            let decoded = decode_jwt(&token).unwrap();

            assert_eq!(decoded.tenant_type, tenant_type);
        }
    }

    // Test empty strings and special characters
    #[tokio::test]
    async fn test_special_characters_in_claims() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        let mut claims = create_test_claims();
        claims.tenant_name = "Test Tenant-Special Chars@#$%".to_string();
        claims.username = "username_with_underscore".to_string();

        let token = create_jwt(&claims).unwrap();
        let decoded = decode_jwt(&token).unwrap();

        assert_eq!(decoded.tenant_name, claims.tenant_name);
        assert_eq!(decoded.username, claims.username);
    }

    // Test various permission list scenarios
    #[tokio::test]
    async fn test_permissions_variations() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");

        // Test single permission
        let mut claims = create_test_claims();
        claims.permissions = vec!["single_permission".to_string()];

        let token = create_jwt(&claims).unwrap();
        let decoded = decode_jwt(&token).unwrap();
        assert_eq!(decoded.permissions, vec!["single_permission"]);

        // Test multiple permissions
        claims.permissions = vec![
            "read".to_string(),
            "write".to_string(),
            "delete".to_string(),
            "admin".to_string(),
        ];

        let token = create_jwt(&claims).unwrap();
        let decoded = decode_jwt(&token).unwrap();
        assert_eq!(decoded.permissions.len(), 4);
        assert!(decoded.permissions.contains(&"read".to_string()));
        assert!(decoded.permissions.contains(&"admin".to_string()));
    }

    // Test JWT operations with custom keys
    #[test]
    fn test_jwt_with_custom_keys() {
        let claims = create_test_claims();
        let keys = create_test_keys("custom_secret_key");

        // Create token
        let token = create_jwt_with_key(&claims, &keys).unwrap();
        assert!(!token.is_empty());

        // Decode token
        let decoded = decode_jwt_with_key(&token, &keys).unwrap();
        assert_eq!(decoded.username, claims.username);
        assert_eq!(decoded.tenant_hash, claims.tenant_hash);
    }

    // Test expiration time edge cases
    #[test]
    fn test_expiration_edge_cases() {
        let keys = create_test_keys("test_secret");

        // Test obviously expired token (expired 5 minutes ago)
        let mut claims = create_test_claims();
        claims.exp = (Utc::now().timestamp() - 300) as usize; // Expired 5 minutes ago

        let token = create_jwt_with_key(&claims, &keys).unwrap();
        let result = decode_jwt_with_key(&token, &keys);
        assert!(result.is_err());

        // Test far future expiration time
        claims.exp = (Utc::now().timestamp() + 86400 * 365) as usize; // Expires in 1 year
        let token = create_jwt_with_key(&claims, &keys).unwrap();
        let result = decode_jwt_with_key(&token, &keys);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_permission_success() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: false,
                path_pattern: "/api/orders".to_string(),
            };
            trie.insert("/api/orders", rule);
            tries.insert(Method::GET, trie);
        }

        let claims = create_test_claims(); // Has "order:read" permission
        let path = "/api/orders";
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_permission_forbidden() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::DELETE,
                required_permission: "order:delete".to_string(), // User doesn't have this permission
                self_only: false,
                path_pattern: "/api/orders".to_string(),
            };
            trie.insert("/api/orders", rule);
            tries.insert(Method::DELETE, trie);
        }

        let claims = create_test_claims(); // Only has "order:read" and "order:write"
        let path = "/api/orders";
        let method = &Method::DELETE;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_verify_access_complex_route_with_multiple_params() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie with complex route
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: true,
                path_pattern: "/api/tenants/{tenant_hash}/users/{username}/orders".to_string(),
            };
            trie.insert("/api/tenants/{tenant_hash}/users/{username}/orders", rule);
            tries.insert(Method::GET, trie);
        }

        let claims = create_test_claims();
        let path = "/api/tenants/test_provider_hash/users/test_user/orders";
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_complex_route_wrong_tenant() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie with complex route
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: true,
                path_pattern: "/api/tenants/{tenant_hash}/users/{username}/orders".to_string(),
            };
            trie.insert("/api/tenants/{tenant_hash}/users/{username}/orders", rule);
            tries.insert(Method::GET, trie);
        }

        let claims = create_test_claims();
        let path = "/api/tenants/wrong_tenant_hash/users/test_user/orders";
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_verify_access_complex_route_wrong_user() {
        setup_test_environment().await;
        use crate::libs::permission::{PermissionRule, PermissionTrie};

        // Setup permission trie with complex route
        {
            let mut tries = PERMISSION_TRIES.write().await;
            tries.clear();

            let mut trie = PermissionTrie::new();
            let rule = PermissionRule {
                method: Method::GET,
                required_permission: "order:read".to_string(),
                self_only: true,
                path_pattern: "/api/tenants/{tenant_hash}/users/{username}/orders".to_string(),
            };
            trie.insert("/api/tenants/{tenant_hash}/users/{username}/orders", rule);
            tries.insert(Method::GET, trie);
        }

        let claims = create_test_claims();
        let path = "/api/tenants/test_provider_hash/users/wrong_user/orders";
        let method = &Method::GET;

        let result = verify_access(path, method, &claims).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }
}
