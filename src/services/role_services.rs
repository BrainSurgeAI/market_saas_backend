use crate::{
    common::{ApiResponse, AppError},
    dto::ValidatedJSON,
    middleware::context::RequestContext,
    models::role::{Permission, Role, UpdateRolePermissionDTO},
    repositories::role_traits::RoleRepository,
    utils::validate_json_fmt::Json,
};
use axum::{extract::Path, Extension};

pub async fn get_roles_by_tenant_type<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path((_, tenant_type)): Path<(String, String)>,
) -> Result<Json<ApiResponse<Vec<Role>>>, AppError>
where
    T: RoleRepository + Send + Sync,
{
    let roles = repo.get_roles_by_tenant_type(&tenant_type).await?;
    Ok(Json(ApiResponse::new(Some(roles), &context)))
}

/// Get all roles, only for super admin
pub async fn get_roles<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
) -> Result<Json<ApiResponse<Vec<Role>>>, AppError>
where
    T: RoleRepository + Send + Sync,
{
    let roles = repo.get_all_roles().await?;
    Ok(Json(ApiResponse::new(Some(roles), &context)))
}

pub async fn get_permissions<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
) -> Result<Json<ApiResponse<Vec<Permission>>>, AppError>
where
    T: RoleRepository + Send + Sync,
{
    let permissions = repo.get_permissions().await?;
    Ok(Json(ApiResponse::new(Some(permissions), &context)))
}

pub async fn get_permissions_by_role_id<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(role_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<Permission>>>, AppError>
where
    T: RoleRepository + Send + Sync,
{
    let permissions = repo.get_permissions_by_role_id(role_id).await?;
    Ok(Json(ApiResponse::new(Some(permissions), &context)))
}

pub async fn update_role_by_id<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(role_id): Path<i32>,
    ValidatedJSON(payload): ValidatedJSON<Role>,
) -> Result<Json<ApiResponse<Role>>, AppError>
where
    T: RoleRepository + Send + Sync,
{
    if payload.id != role_id || payload.id == 1 {
        return Err(AppError::Forbidden("超级管理员角色不能修改".to_string()));
    }

    repo.update_role_by_id(&payload).await?;
    Ok(Json(ApiResponse::new(Some(payload), &context)))
}

/// 更新角色权限
///
/// 此函数接收角色ID和权限ID列表，替换该角色的所有权限
pub async fn update_role_permissions<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(role_id): Path<i32>,
    Json(payload): Json<UpdateRolePermissionDTO>,
) -> Result<Json<ApiResponse<()>>, AppError>
where
    T: RoleRepository + Send + Sync,
{
    if role_id == 1 {
        return Err(AppError::Forbidden("超级管理员角色不能修改".to_string()));
    }

    repo.update_role_permissions(role_id, &payload.permissions)
        .await?;
    Ok(Json(ApiResponse::new(Some(()), &context)))
}

pub async fn get_role_by_id<T>(
    Extension(repo): Extension<T>,
    Extension(context): Extension<RequestContext>,
    Path(role_id): Path<i32>,
) -> Result<Json<ApiResponse<Role>>, AppError>
where
    T: RoleRepository + Send + Sync,
{
    let role = repo.get_role_by_id(role_id).await?;
    Ok(Json(ApiResponse::new(Some(role), &context)))
}
