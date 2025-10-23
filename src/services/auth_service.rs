use crate::common::{ApiResponse, AppError};
use crate::dto::auth::{LoginRequest, RegisterRequest, ResetPasswordRequest};
use crate::dto::ValidatedJSON;
use crate::middleware::context::RequestContext;
use crate::models::claims::Claims;
use crate::repositories::tenant_name_hash;
use crate::repositories::tenants_trait::TenantRepository;
use crate::utils::validate_json_fmt::Json;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Extension;

use tracing::{debug, error, info, warn};

use validator::Validate;

use crate::middleware::auth::create_jwt;
use crate::repositories::user_traits::{CreateTenantWithAdminParams, UserRepository};
use crate::utils::validator::{check_password, validate_username};

/// Register a new tenant and admin user.
///
/// Only the admin user can register a new tenant,
/// and the admin user will be created by the system automatically.
/// The other users can be created by the admin user.
///
/// # Arguments
//  * `repo` - Repository implementation for user and tenant operations

/// * `payload` - Registration request containing:
///   * `username` - Username for admin account (6-32 chars)
///   * `password` - Password for admin account (8-16 chars, must contain at least one uppercase letter, one lowercase letter, one digit, and one special character)
///   * `tenant_type` - Type of tenant (4-16 chars) Only "CUSTOMER", "PROVIDER", "MARKET" are allowed
///   * `tenant_name` - Name of tenant (2-255 chars)
///
/// # Flow
/// 1. Validates registration payload
/// 2. Checks if username already exists
/// 3. Checks if tenant name already exists  
/// 4. Creates tenant and admin user with appropriate role
/// 5. Returns success response
///
/// # Errors
/// * `400 Bad Request` - If validation fails for the request payload
/// * `409 Conflict` - If username already exists or tenant name already exists
/// * `500 Internal Server Error` - If server error occurs during registration
///
/// # Returns
/// - 201 Created: Tenant and admin user created successfully
/// - Error response with appropriate status code and message
///
/// # Example Request
/// ```json
/// {
///   "username": "john.doe",
///   "password": "passWord123!",
///   "tenant_type": "CUSTOMER",
///   "tenant_name": "John's Store"
/// }   
/// ```
///
/// # Example Success Response
/// ```json
/// {
///   "code": 201,
///   "message": "success",
///   "data": "eyJhbGciOiJIUzI1NiIs...",
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
///
/// # Example Error Response    
/// ```json
/// {
///   "code": 400,
///   "message": "Invalid username or password", or "Invalid tenant type",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///
/// }
/// ```
///
/// # Example Error Response    
/// ```json
/// {
///   "code": 409,
///   "message": "Username already exists" or "Tenant already exists",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
///
/// # Example Error Response    
/// ```json
/// {
///   "code": 500,
///   "message": "Internal server error",   
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```

#[utoipa::path(
    post,
    path = "/api/v1/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Register successful", body = ApiResponse<String>),
        (status = 400, description = "Username or password is invalid", body = ApiResponse<String>),
        (status = 409, description = "Username already exists", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    tag = "User Registration"
)]
pub async fn register<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError>
where
    T: UserRepository + TenantRepository + Send + Sync,
{
    validate_registration(&payload).await?;

    if repo
        .is_user_exists(&payload.username)
        .await?
        .then_some(())
        .is_some()
    {
        warn!("Username already exists: {}", payload.username);
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    if repo
        .is_tenant_exist(&payload.tenant_name)
        .await?
        .then_some(())
        .is_some()
    {
        warn!("Tenant already exists: {}", payload.tenant_name);
        return Err(AppError::Conflict("Tenant already exists".to_string()));
    }

    // payload.tenant_type is like "CUSTOMER", so we need to convert it to "CUSTOMER_ADMIN"
    let mut roles = String::with_capacity(payload.tenant_type.len() + 6); // "_ADMIN" is 6 chars
    roles.push_str(&payload.tenant_type.to_uppercase());
    roles.push_str("_ADMIN");
    debug!("roles: {}", roles);

    let hashed_password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST).map_err(|e| {
        error!("Error hashing password: {:?}", e);
        AppError::Internal("Error hashing password".to_string())
    })?;

    let tenant_hash_name = tenant_name_hash(&payload.tenant_name).unwrap();
    let tenant_admin_params = CreateTenantWithAdminParams::builder()
        .tenant_type(payload.tenant_type.to_uppercase())
        .tenant_name(&payload.tenant_name)
        .tenant_hash_name(&tenant_hash_name)
        .username(&payload.username)
        .name(&payload.name)
        .hashed_password(&hashed_password)
        .role(&roles)
        .build()
        .unwrap();

    let user_id = create_tenant_with_admin_user(&repo, &tenant_admin_params).await?;

    let permissions = repo.get_permissions_by_role(&roles).await?;

    if let Some(_permissions) = permissions {
        

        let claims = Claims {
            tenant_type: payload.tenant_type.to_uppercase(),
            tenant_name: payload.tenant_name,
            tenant_hash: tenant_hash_name,
            username: payload.username,
            roles: vec![roles],
            is_super_admin: false,
            exp: chrono::Utc::now()
                .checked_add_signed(chrono::Duration::days(1))
                .expect("Invalid timestamp")
                .timestamp() as usize,
        };

        let token = create_jwt(&claims).map_err(|e| {
            error!("Error creating JWT: {:?}", e);
            AppError::Internal("Error creating JWT".to_string())
        })?;

        send_welcome_message(&repo, user_id, &claims.username).await?;
        Ok((
            axum::http::StatusCode::CREATED,
            Json(ApiResponse::new(Some(token), &context)),
        ))
    } else {
        warn!(
            "No permissions found for {} with role: {}",
            &payload.username, &roles
        );
        Err(AppError::Internal("Error creating JWT".to_string()))
    }
}

/// Login with username and password
///
/// # Description
/// Authenticates a user with their username and password and returns a JWT token.
/// The token contains the user's roles and permissions.
///
/// # Arguments
//  * `repo` - Repository implementation for user and tenant operations
/// * `payload` - Login request containing:
///   * `username` - Username for admin account (6-32 chars)
///   * `password` - Password for admin account (8-16 chars)
///
/// # Flow
/// 1. Validates the request payload
/// 2. Retrieves user permissions and password hash from database
/// 3. Verifies the password using bcrypt
/// 4. Generates a JWT token containing roles and permissions
///
/// # Errors
/// - 400 Bad Request: Invalid request payload
/// - 401 Unauthorized: Invalid username or password
/// - 404 Not Found: User not found or no permission found for this user
/// - 500 Internal Server Error: Error creating JWT or database error
///
/// # Returns
/// - 200 OK: JWT token for authenticated user
/// - Error response with appropriate status code and message
///
/// # Example Request
/// ```json
/// {
///   "username": "john.doe",
///   "password": "passWord123!"
/// }
/// ```
///
/// # Example Success Response
/// ```json
/// {
///   "code": 200,
///   "message": "success",
///   "data": "eyJhbGciOiJIUzI1NiIs..."
/// }   
/// ```
///
/// # Example Error Response
/// ```json
/// {
///   "code": 400,
///   "message": "Invalid request",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
///
/// # Example Error Response
/// ```json
/// {
///   "code": 401,
///   "message": "Invalid credentials",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
///
/// # Example Error Response
/// ```json
/// {
///   "code": 404,
///   "message": "Invalid username or password",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
///
/// # Example Error Response
/// ```json
/// {
///   "code": 500,
///   "message": "Internal server error",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<String>),
        (status = 400, description = "Invalid request", body = ApiResponse<String>),
        (status = 401, description = "Invalid username or password", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    tag = "User Login"
)]
pub async fn login<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError>
where
    T: UserRepository + TenantRepository + Send + Sync,
{
    if let Err(validation_errors) = payload.validate() {
        warn!("Validation errors: {:?}", validation_errors);
        return Err(AppError::Validation(validation_errors.to_string()));
    }

    let user_auth = repo
        .get_user_permissions(&payload.username)
        .await?
        .ok_or_else(|| {
            warn!(
                "Cannot find user by username: {} or has no permission",
                payload.username
            );
            AppError::Auth("Invalid username or password".to_string())
        })?;

    debug!(
        "User {} roles: {:?} permissions: {:?}",
        payload.username, user_auth.roles, user_auth.permissions
    );

    if !bcrypt::verify(&payload.password, &user_auth.password_hash).map_err(|e| {
        error!("bcrypt verify error: {}", e);
        AppError::Internal("Password verification failed".to_string())
    })? {
        warn!(
            "Invalid username {} or password {}",
            payload.username, payload.password
        );
        return Err(AppError::Auth("Invalid username or password".to_string()));
    }


    let tenant = repo
        .find_tenant_by_username(&payload.username)
        .await?
        .ok_or(AppError::Auth("Invalid username or password".to_string()))?;

    let claims = Claims {
        tenant_type: tenant.tenant_type,
        tenant_name: tenant.name,
        tenant_hash: tenant.name_hash,
        username: payload.username,
        roles: vec![user_auth.roles.to_string()],
        is_super_admin: user_auth.is_super_admin,
        exp: chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(1))
            .expect("Invalid timestamp")
            .timestamp() as usize,
    };

    let token = create_jwt(&claims).map_err(|e| {
        error!("Error creating JWT: {:?}", e);
        AppError::Internal("Error creating JWT".to_string())
    })?;

    info!("User {} login success", claims.username);
    Ok(Json(ApiResponse::new(Some(token), &context)))
}

/// Reset a user's password.
///
/// Allows a user to reset their password by providing their current password and a new password.
/// The new password must meet the password requirements.
///
/// # Arguments
// * `repo` - Repository implementation for user operations
// * `context` - Request context containing request ID and timestamp
/// * `payload` - Reset password request containing:
///   * `current_password` - Current password for verification
///   * `new_password` - New password (8-16 chars, must contain at least one uppercase letter, one lowercase letter, one digit, and one special character)
///
/// # Flow
/// 1. Validates reset password payload
/// 2. Verifies current password matches
/// 3. Updates password to new password
/// 4. Returns success response
///
/// # Errors
/// * `400 Bad Request` - If validation fails for the new password
/// * `401 Unauthorized` - If current password is incorrect
/// * `404 Not Found` - If username does not exist
/// * `500 Internal Server Error` - If server error occurs during password reset
///
/// # Returns
/// - 200 OK: Password successfully reset
/// - Error response with appropriate status code and message
///
/// # Example Request
/// ```json
/// {
///   "current_password": "oldPass123!",
///   "new_password": "newPass456!"
/// }
/// ```
///
/// # Example Success Response
/// ```json
/// {
///   "code": 200,
///   "message": "success",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
///
/// # Example Error Response
/// ```json
/// {
///   "code": 400,
///   "message": "Invalid password format",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
///
/// # Example Error Response
/// ```json
/// {
///   "code": 401,
///   "message": "Current password is incorrect",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```

#[utoipa::path(
    post,
    path = "/api/v1/users/{username}/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = ApiResponse<String>),
        (status = 400, description = "Invalid payload", body = ApiResponse<String>),
        (status = 401, description = "Current password is incorrect", body = ApiResponse<String>),
        (status = 404, description = "User not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    tag = "Users"
)]
pub async fn reset_password<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(username): Path<String>,
    ValidatedJSON(payload): ValidatedJSON<ResetPasswordRequest>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: UserRepository + Send + Sync,
{
    repo.reset_password(&username, &payload.current_password, &payload.new_password)
        .await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

/// private functions

async fn validate_registration(payload: &RegisterRequest) -> Result<(), AppError> {
    // validation for fileds length
    if let Err(validation_errors) = payload.validate() {
        error!("Validation errors: {:?}", validation_errors);
        return Err(AppError::Validation(validation_errors.to_string()));
    }

    // validate formats of username, password
    if !validate_username(&payload.username)
        || !check_password(&payload.password)
        || payload.tenant_type.is_empty()
    {
        error!("Invalid username or password");
        return Err(AppError::Validation(
            "Invalid username or password".to_string(),
        ));
    }

    // validate tenant type
    let tenant_type = payload.tenant_type.to_uppercase();
    if tenant_type != "CUSTOMER" && tenant_type != "PROVIDER" && tenant_type != "MARKET" {
        error!("Invalid tenant type");
        return Err(AppError::Validation("Invalid tenant type".to_string()));
    }
    Ok(())
}

async fn send_welcome_message<T>(repo: &T, user_id: u64, username: &str) -> Result<(), AppError>
where
    T: UserRepository + Send + Sync,
{
    let message_content = format!("欢迎, {}! 您已成功注册，请先完善您的信息", username);
    send_message_to_user(repo, user_id, &message_content).await
}

// TODO: send message to message queue
// Example: send_to_message_queue(user_id, content).await?;
async fn send_message_to_user<T>(repo: &T, user_id: u64, content: &str) -> Result<(), AppError>
where
    T: UserRepository + Send + Sync,
{
    repo.send_message_to_user(user_id, content).await?;
    info!(
        "Message sent to user: {} with content: {}",
        user_id, content
    );
    Ok(())
}

async fn create_tenant_with_admin_user<T>(
    repo: &T,
    params: &CreateTenantWithAdminParams,
) -> Result<u64, AppError>
where
    T: UserRepository + Send + Sync,
{
    repo.create_tenant_with_admin(params).await
}
