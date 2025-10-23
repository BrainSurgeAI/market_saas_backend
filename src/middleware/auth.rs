use crate::repositories::tenants_trait::TenantRepository;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use std::sync::LazyLock;

use crate::{acl_core::acl_snapshot::ACL_SNAPSHOT, models::claims::Claims};
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use hyper::{header, StatusCode};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use tracing::{debug, error, warn};

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

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn TenantRepository + Send + Sync>,
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
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let path = req.uri().path();
    let method = req.method();

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

    // Decode and validate JWT token
    let claims = decode_jwt(token).map_err(|_| {
        warn!("Invalid JWT token for {} {}", method, path);
        StatusCode::UNAUTHORIZED
    })?;

    if !claims.roles.contains(&"SUPER_ADMIN".to_string()) {
        let tenant_exists = state
            .repo
            .verify_tenant_exists(&claims.tenant_hash)
            .await
            .map_err(|e| {
                error!(
                    "Failed to verify tenant existence for {}: {}",
                    claims.username, e
                );
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if !tenant_exists {
            warn!(
                "Tenant does not exist for user {} with tenant_hash {}",
                claims.username, claims.tenant_hash
            );
            return Err(StatusCode::FORBIDDEN);
        }

        debug!(
            "tenant is exists {} for user {} with tenant_hash {}",
            tenant_exists, claims.username, claims.tenant_hash
        );
    }

    // Verify access permissions
    ACL_SNAPSHOT
        .load()
        .verify_access(method, path, &claims)
        .await
        .map_err(|status| {
            warn!(
                "Access denied for user {} to {} {}: {:?}",
                claims.username, method, path, status
            );
            status
        })?;

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

    // 4. Inject claims into request extensions
    req.extensions_mut().insert(claims.clone());

    debug!(
        "ok, this request is authenticated for user {}",
        claims.username
    );
    Ok(next.run(req).await)
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
