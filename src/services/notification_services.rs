use crate::common::{ApiResponse, AppError};
use crate::middleware::context::RequestContext;
use crate::models::claims::Claims;
use crate::services::message::{MessageDto, MessageQueryParams, PaginatedMessages};
use axum::{extract::Query, extract::Path, Extension, Json};
use sqlx::MySqlPool;

pub(crate) async fn list_notifications(
    Extension(pool): Extension<MySqlPool>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<MessageQueryParams>,
) -> Result<Json<ApiResponse<PaginatedMessages>>, AppError> {
    let messages = MessageDto::get_notifications(&pool, &claims.username, Some(&params)).await?;
    Ok(Json(ApiResponse::new(Some(messages), &context)))
}

pub(crate) async fn read_notification(
    Extension(pool): Extension<MySqlPool>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
    Path(message_id): Path<i32>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    let res = MessageDto::update_message_to_read(&pool, message_id, &claims.username).await?;
    Ok(Json(ApiResponse::new(Some(res), &context)))
}
