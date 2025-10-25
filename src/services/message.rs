use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::error;
use crate::{common::AppError, map_db_err};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct MessageDto {
    pub(crate) id: i32,
    pub(crate) user_id: i32,
    pub(crate) content: String,
    pub(crate) is_read: bool,

    #[serde(with = "chrono::serde::ts_seconds")]
    pub(crate) created_at: DateTime<Utc>,
}

impl MessageDto {
    pub(crate) async fn get_unread_messages(
        pool: &sqlx::MySqlPool,
        username: &str,
    ) -> Result<Vec<MessageDto>, AppError> {
        let messages = sqlx::query_as::<_, MessageDto>(
            "SELECT m.* FROM messages m 
         INNER JOIN users u ON m.user_id = u.id 
         WHERE u.username = ? AND m.is_read = false 
         ORDER BY m.created_at DESC",
        )
        .bind(username)
        .fetch_all(pool)
        .await
        .map_err(map_db_err!("Failed to fetch unread messages"))?;

        Ok(messages)
       
    }

    pub(crate) async fn update_message_to_read(
        pool: &sqlx::MySqlPool,
        message_id: i32,
        username: &str
    ) -> Result<bool, sqlx::Error> {
        let affected = sqlx::query(
            r#"UPDATE messages m SET m.is_read = true 
                   WHERE m.id = ? AND m.user_id = (SELECT u.id FROM users u where u.username = ? )"#)
            .bind(message_id)
            .bind(username)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("Failed to mark message as read: {}", e);
                e
            })?;

        Ok(affected.rows_affected() > 0)
    }
}
