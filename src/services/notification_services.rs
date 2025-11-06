use crate::common::{ApiResponse, AppError};
use crate::middleware::context::RequestContext;
use crate::models::claims::Claims;
use crate::services::message::MessageDto;
use axum::{extract::Path, Extension, Json};
use sqlx::MySqlPool;

pub(crate) async fn list_notifications(
    Extension(pool): Extension<MySqlPool>,
    Extension(context): Extension<RequestContext>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<MessageDto>>>, AppError> {
    let messages = MessageDto::get_unread_messages(&pool, &claims.username).await?;
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
