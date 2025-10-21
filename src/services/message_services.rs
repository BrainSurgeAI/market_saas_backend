use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::error;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: i32,
    pub user_id: i32,
    pub content: String,
    pub is_read: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub async fn get_unread_messages(
        pool: &sqlx::MySqlPool,
        username: &str,
    ) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            "SELECT m.* FROM messages m 
         INNER JOIN users u ON m.user_id = u.id 
         WHERE u.username = ? AND m.is_read = false 
         ORDER BY m.created_at DESC",
        )
        .bind(username)
        .fetch_all(pool)
        .await
    }

    pub async fn mark_as_read(
        pool: &sqlx::MySqlPool,
        message_id: i32,
    ) -> Result<bool, sqlx::Error> {
        let affected = sqlx::query("UPDATE messages m SET m.is_read = true WHERE m.id = ?")
            .bind(message_id)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("Failed to mark message as read: {}", e);
                e
            })?;

        Ok(affected.rows_affected() > 0)
    }
}
