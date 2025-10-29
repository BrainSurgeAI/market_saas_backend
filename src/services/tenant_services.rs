use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{extract::Path, Extension};
use tracing::debug;

use crate::common::{ApiResponse, AppError};

use crate::dto::ValidatedJSON;
use crate::middleware::context::RequestContext;
use crate::models::claims::Claims;

use crate::repositories::tenants_trait::TenantRepository;

use crate::dto::tenants::{
    BaseTenantDTO, Provider, QueryTenantByType, TenantAddressUpdate, TenantCreateDTO,
};
use crate::utils::validate_json_fmt::Json;


/// Retrieves a tenant by username.
///
/// # Arguments
/// * `repo` - Repository implementation for tenant operations
/// * `username` - Username of the user to retrieve tenant for
///
/// # Returns
/// Returns a JSON response containing:
/// * On success: Status 200 with the tenant details
/// * On DB error: Status 500 for database errors
/// * On other errors: Status 417 for unexpected errors
///
/// # Example Success Response
/// ```json
/// {
///     "code": 200,
///     "message": "Success",
///     "data": {
///         "id": 1,
///         "name": "Test Store",
///         "address": "Test Address",
///         "phone": "1234567890",
///         "license_image": "/images/license/1.png" or not returned,
///         "status": "PENDING",
///         "created_at": "2025-02-04T10:24:33Z",     
///         "updated_at": 2025-02-04T10:24:33Z
///     }
/// }
/// ```
#[utoipa::path(
    get,
    path = "/api/users/{username}/tenants",
    responses(
        (status = 200, description = "Get tenant by username successful", body = BaseTenantDTO),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Invalid request")
    ),
    tag = "Tenants"
)]

pub async fn get_tenant_by_user<T>(
    Extension(context): Extension<RequestContext>,
    Extension(repo): Extension<T>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    let tenants = repo.find_tenant_by_username(&username).await?;
    Ok(Json(ApiResponse::new(Some(tenants), &context)))
}

// Create tenant by market
pub async fn create_tenant<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(market_hash): Path<String>,
    ValidatedJSON(payload): ValidatedJSON<TenantCreateDTO>,
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    debug!("Creating tenant for {:?}", payload);
    if claims.roles[0].to_uppercase() != "MARKET_ADMIN" {
        return Err(AppError::Forbidden(
            "Only market admin can create tenant".to_string(),
        ));
    }

    if payload.tenant_type.to_uppercase() == "MARKET" {
        return Err(AppError::Validation(
            "Only super admin can create market".to_string(),
        ));
    }

    repo.create_tenant(&market_hash, &payload).await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

pub async fn list_tenants_by_market<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<QueryTenantByType>,
) -> Result<Json<ApiResponse<Vec<BaseTenantDTO>>>, AppError>
where
    T: TenantRepository + Send + Sync,
{
    if claims.roles[0].to_uppercase() != "MARKET_ADMIN" {
        return Err(AppError::Forbidden(
            "Only market admin can list tenants".to_string(),
        ));
    }

    let res = repo
        .get_all_tenants_by_market(&claims.tenant_hash, &query)
        .await?;
    Ok(Json(ApiResponse::new(Some(res), &context)))
}

pub async fn get_tenant_financials<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    let financials = repo.get_tenant_financials(&claims.tenant_hash).await?;
    Ok(Json(ApiResponse::new(financials, &context)))
}

pub(crate) async fn get_tenant_users<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(hashed_name): Path<String>,
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    let users = repo.get_tenant_users(&hashed_name).await?;
    Ok(Json(ApiResponse::new(Some(users), &context)))
}

pub(crate) async fn get_users<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    let users = repo.get_tenant_users(&claims.tenant_hash).await?;
    Ok(Json(ApiResponse::new(Some(users), &context)))
}


/// Add a user to a tenant
///
/// Adds a user to a tenant by their username and hashed name.
///
/// Required permission: tenant:create,user:read
///
/// # Arguments
/// * `hashed_name` - Hashed name of the tenant
/// * `payload` - User details to add to the tenant
///    username: String,
///    email: String,
///    phone: String,
///    roles: [i32]
///
/// # Returns
/// * `200 OK` - Returns the user ID of the added user
// pub async fn add_user_to_tenant<T>(
//     Extension(repo): Extension<T>,
//     Extension(context): Extension<RequestContext>,
//     Path(hashed_name): Path<String>,
//     Json(payload): Json<User>,
// ) -> Result<impl IntoResponse, AppError>
// where
//     T: TenantRepository + Send + Sync,
// {
//     debug!("Adding user to tenant: {} roles: {:?}", hashed_name, payload.roles);

//     let user_id = repo.add_user_to_tenant(&hashed_name, &payload).await?;
//     Ok(Json(ApiResponse::new(Some(user_id), &context)))
// }

/// Update a tenant
///
/// Updates a tenant by their hashed name and tenant details.
///
/// Required permission: tenant:create,tenant:update
///
/// # Arguments
/// * `hashed_name` - Hashed name of the tenant
/// * `payload` - Tenant details to update
///
/// # Returns
/// * `200 OK` - Returns the number of rows updated
///
pub async fn update_tenant_by_self<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(hashed_name): Path<String>,
    Json(payload): Json<TenantAddressUpdate>,
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    let rows_affected = repo.update_tenant_self(&hashed_name, &payload).await?;
    Ok(Json(ApiResponse::new(Some(rows_affected), &context)))
}

/// Get all providers by market
///
/// Retrieves all providers by market hash.
///
/// Required permission: tenant:create,tenant:update
///
/// # Arguments
/// * `market_hash` - Hashed name of the market
///
/// # Returns
/// * `200 OK` - Returns all providers by market hash
/// * `404 Not Found` - If market is not found
/// * `500 Internal Server Error` - If there was a server error
#[utoipa::path(
    get,
    path = "/api/v1/markets/{market_hash}/providers",
    params(
        ("market_hash" = String, Path, description = "Market identifier")
    ),
    responses(
        (status = 200, description = "Get all providers by market hash successful", body = ApiResponse<Vec<Provider>>),
        (status = 404, description = "Market not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tenants"
)]
pub(crate) async fn get_all_providers_by_market<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    let providers = repo
        .get_all_providers_by_market(&claims.tenant_hash)
        .await?;
    Ok(Json(ApiResponse::new(Some(providers), &context)))
}

pub async fn disable_tenant<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path((market_hash, tenant_hash)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    if claims.roles[0].to_uppercase() != "MARKET_ADMIN" {
        return Err(AppError::Forbidden(
            "Only market admin can disable tenant".to_string(),
        ));
    }

    let rows_affected = repo.delete_tenant(&market_hash, &tenant_hash).await?;
    Ok(Json(ApiResponse::new(Some(rows_affected), &context)))
}

/// Market Admin updates tenants that belongs to market
pub async fn update_tenant_by_market<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(tenant_hash): Path<String>,
    ValidatedJSON(payload): ValidatedJSON<TenantCreateDTO>,
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    let rows_affected = repo
        .update_tenant_by_market(&claims.tenant_hash, &tenant_hash, &payload)
        .await?;
    Ok(Json(ApiResponse::new(Some(rows_affected), &context)))
}

/// Get tenant detail by hashed name of tenant.
/// Use hashed name from JWT claims, so it is used for tenant to get its own detail.
/// # Arguments
/// * `repo` - Repository implementation for tenant operations
/// * `claims` - Claims extracted from JWT token
/// # Returns
/// Returns a JSON response containing:
/// * On success: Status 200 with the tenant details
/// * On DB error: Status 500 for database errors
/// * On other errors: Status 417 for unexpected errors
pub async fn get_tenant_detail_by_self<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    let tenant = repo
        .find_tenant_detail_by_hashed_name(&claims.tenant_hash)
        .await?;
    Ok(Json(ApiResponse::new(tenant, &context)))
}


pub async fn get_tenant_detail_by_hashed_name<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(tenant_hash): Path<String>
) -> Result<impl IntoResponse, AppError>
where
    T: TenantRepository + Send + Sync,
{
    let tenant = repo
        .find_tenant_detail_by_hashed_name(&tenant_hash)
        .await?;
    Ok(Json(ApiResponse::new(tenant, &context)))
}

