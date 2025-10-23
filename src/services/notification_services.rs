use crate::services::message_services::Message;
use axum::{extract::Path, http::StatusCode, Extension, Json};
use serde::Serialize;
use sqlx::MySqlPool;
use tracing::{debug, error};
use crate::models::claims::Claims;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    code: u16,
    message: String,
    data: T,
}

pub async fn list_notifications(
    Extension(claims): Extension<Claims>,
    Extension(pool): Extension<MySqlPool>,
) -> Json<ApiResponse<Vec<Message>>> {
    match Message::get_unread_messages(&pool, &claims.username).await {
        Ok(messages) => Json(ApiResponse {
            code: StatusCode::OK.into(),
            message: "Success".to_string(),
            data: messages,
        }),
        Err(e) => {
            error!("Failed to fetch messages: {}", e);
            Json(ApiResponse {
                code: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "Failed to fetch messages".to_string(),
                data: vec![],
            })
        }
    }
}

pub async fn update_notification_status(
    Path((_username, message_id)): Path<(String, i32)>,
    Extension(pool): Extension<MySqlPool>,
) -> Json<ApiResponse<bool>> {
    debug!("Marking message {} as read", message_id);

    match Message::mark_as_read(&pool, message_id).await {
        Ok(result) => Json(ApiResponse {
            code: StatusCode::OK.into(),
            message: "Success".to_string(),
            data: result,
        }),
        Err(e) => {
            error!("Failed to mark as read: {}", e);
            Json(ApiResponse {
                code: StatusCode::INTERNAL_SERVER_ERROR.into(),
                message: "Failed to mark as read".to_string(),
                data: false,
            })
        }
    }
}
