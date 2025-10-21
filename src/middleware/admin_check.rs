use crate::{common::AppError, models::claims::Claims};
use axum::{extract::Request, middleware::Next, response::Response, Extension};

/// 检查用户是否为超级管理员的中间件
pub async fn require_super_admin(
    Extension(claims): Extension<Claims>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    if !claims.is_super_admin {
        return Err(AppError::Auth("仅超级管理员可执行此操作".to_string()));
    }

    Ok(next.run(request).await)
}
