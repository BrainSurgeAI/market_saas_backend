use crate::common::{ApiResponse, AppError};
use crate::dto::users::{UserCreateDto, UserResponseDto, UserUpdateDto};
use crate::dto::ValidatedJSON;
use crate::middleware::context::RequestContext;
use crate::models::claims::Claims;
use crate::repositories::user_traits::UserRepository;
use crate::utils::validate_json_fmt::Json;
use axum::extract::Path;
use axum::Extension;
use tracing::debug;

/// Create a user in a tenant.
///
/// # Description
/// Creates a user in a tenant by tenant admin.
///
/// # Arguments
/// * `context` - Request context containing request ID and timestamp
/// * `tenant_hash` - Tenant hash of the tenant to create the user in
/// * `payload` - User create payload containing:
///   * `name` - User's full name (2-64 chars)
///   * `username` - User's username (2-64 chars)
///   * `password` - User's password (8-16 chars)
///   * `role` - User's role (2-64 chars)
pub async fn create_tenant_user<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(tenant_hash): Path<String>,
    ValidatedJSON(payload): ValidatedJSON<UserCreateDto>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: UserRepository + Send + Sync,
{
    debug!("UserCreateDto: {:?}", payload);
    repo.create_tenant_user(&tenant_hash, &payload).await?;
    Ok(Json(ApiResponse::new(None, &context)))
}

/// Update a user's profile.
///
/// # Description
/// Updates a user's profile information by username.
///
/// # Arguments
/// * `context` - Request context containing request ID and timestamp
/// * `username` - Username of the user to update, get from path
/// * `payload` - Profile update containing:
///   * `name` - User's full name (2-64 chars)
///   * `email` - User's email address (valid email format)
///   * `phone` - User's phone number (valid phone format)
///   * `avatar` - User's avatar URL (optional)
///   * `description` - User's description (optional, max 255 chars)
///
/// # Flow
/// 1. Validates profile payload fields
/// 2. Checks if username exists
/// 3. Updates user profile in database
/// 4. Returns success response
///
/// # Errors
/// * `400 Bad Request` - If validation fails for the profile fields
/// * `404 Not Found` - If username does not exist
/// * `409 Conflict` - If email/phone already exists for another user
/// * `500 Internal Server Error` - If server error occurs during update
///
/// # Returns
/// - 200 OK: Profile successfully updated
/// - Error response with appropriate status code and message
///
/// # Example Request
/// ```json
/// {
///   "name": "John Doe",
///   "email": "john.doe@example.com",
///   "phone": "+1234567890",
///   "avatar": "https://example.com/avatar.jpg",
///   "description": "Software Engineer"
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
///   "message": "Invalid email format",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
#[utoipa::path(
    patch,
    path = "/api/v1/users/{username}",
    request_body = UserCreateDto,
    responses(
        (status = 200, description = "Profile updated successfully", body = ApiResponse<String>),
        (status = 400, description = "Invalid payload", body = ApiResponse<String>),
        (status = 404, description = "User not found", body = ApiResponse<String>),
        (status = 409, description = "Email/phone already exists", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Users"
)]

pub async fn update_user<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(username): Path<String>,
    ValidatedJSON(payload): ValidatedJSON<UserUpdateDto>,
) -> Result<Json<ApiResponse<String>>, AppError>
where
    T: UserRepository + Send + Sync,
{
    repo.update_user(&username, &payload).await?;
    Ok(Json(ApiResponse::new(None, &context)))
}

/// Delete a user.
///
/// # Description
/// Deletes a user by username. Update user's deleted_at field to current timestamp.
///
/// # Arguments
// * `repo` - Repository implementation for user operations
// * `context` - Request context containing request ID and timestamp
/// * `username` - Username of the user to update, get from path
///
/// # Flow
/// 1. Validates the user is exists
/// 2. Updates the user was deleted in the database
/// 3. Returns success response
///
/// # Errors
/// * `400 Bad Request` - If validation fails for the profile fields
/// * `404 Not Found` - If username does not exist
/// * `409 Conflict` - If username is already deleted   
/// * `500 Internal Server Error` - If server error occurs during update
///
/// # Returns
/// - 200 OK: User successfully deleted
/// - Error response with appropriate status code and message
///
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
///   "code": 404,
///   "message": "User not found",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
///
#[utoipa::path(
    patch,
    path = "/api/v1/tenants/{hashed_name}/users/{username}",
    params(
        ("hashed_name" = String, Path, description = "Hashed name of the tenant"),
        ("username" = String, Path, description = "Username of the user to update")
    ),
    responses(
        (status = 200, description = "User disabled successfully", body = ApiResponse<String>),
        (status = 404, description = "User not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    tag = "Users"
)]
pub async fn delete_user<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path((_hashed_name, username)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: UserRepository + Send + Sync,
{
    if claims.roles.iter().any(|role| role.contains("ADMIN")) {
        repo.disable_user(&username).await?;
        Ok(Json(ApiResponse::new(None, &context)))
    } else {
        Err(AppError::Forbidden(
            "You are not allowed to disable user".to_string(),
        ))
    }
}

/// Enable a user.
///
/// # Description
/// Enables a user by username. Update user's deleted_at field to null.
///
/// # Arguments
/// * `repo` - Repository implementation for user operations
/// * `context` - Request context containing request ID and timestamp
/// * `username` - Username of the user to enable
///

pub async fn enable_user<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path((_hashed_name, username)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: UserRepository + Send + Sync,
{
    repo.enable_user(&username).await?;
    Ok(Json(ApiResponse::new(None, &context)))
}

/// Get user profile by username.
///
/// Retrieves a user's profile information based on their username.
///
/// # Arguments
///* `repo` - Repository implementation for user operations
// * `context` - Request context containing metadata
/// * `username` - Username of the user to retrieve
///
/// # Flow
/// 1. Looks up user profile by username in repository
/// 2. Returns profile if found
/// 3. Returns appropriate error if not found or other error occurs
///
/// # Errors
/// * `404 Not Found` - If user profile does not exist
/// * `500 Internal Server Error` - If server error occurs during lookup
///
/// # Returns
/// - 200 OK with user profile data
/// - Error response with appropriate status code and message
///
/// # Example Success Response
/// ```json
/// {
/// "code":200,
/// "message":"success",
///   "data":{
///           "username":"ryman1981",
///           "email": "ryman1981@163.com",
///           "phone": "13800138000",
///           "tenant_id":1,
///           "tenant_name":"新疆新联大市场",
///           "created_at":1739706295,
///           "updated_at":1739712417,
///           "deleted_at":null
///   },
///   "requestId":"385afe60-d2ed-43fd-90a2-f2fcad38c82c",
///   "timestamp":"2025-02-16T16:04:01.815473136Z"
/// }
/// ```
///
/// # Example Error Response
/// ```json
/// {
///   "code": 404,
///   "message": "Profile not found",
///   "data": null,
///   "requestId": "123e4567-e89b-12d3-a456-426614174000",
///   "timestamp": "2023-01-01T00:00:00Z"
/// }
/// ```
///
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = ApiResponse<UserCreateDto>),
        (status = 404, description = "User profile not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    tag = "Users"
)]

pub async fn get_user<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    // Path(username): Path<String>,
) -> Result<Json<ApiResponse<UserResponseDto>>, AppError>
where
    T: UserRepository + Send + Sync,
{
    let profile = repo.get_user_by_username(&claims.username).await?;
    Ok(Json(ApiResponse::new(profile, &context)))
}
