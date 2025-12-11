use crate::{common::AppError, map_db_err};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::error;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct MessageDto {
    pub(crate) id: u64,

    #[serde(rename = "userId")]
    pub(crate) user_id: i32,
    pub(crate) category: String,
    pub(crate) title: String,
    pub(crate) content: String,

    #[serde(rename = "isRead")]
    pub(crate) is_read: bool,

   // #[serde(with = "chrono::serde::ts_seconds")]
    #[serde(rename = "createdAt")]
    pub(crate) created_at: DateTime<Utc>,
}

/// 消息查询参数
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(crate) struct MessageQueryParams {
    /// 页码，从1开始
    #[serde(rename = "page")]
    pub(crate) page: Option<i32>,

    /// 每页大小
    #[serde(rename = "pageSize")]
    pub(crate) page_size: Option<i32>,

    /// 是否已读过滤：true=只返回已读，false=只返回未读，None=返回全部
    #[serde(rename = "isRead")]
    pub(crate) is_read: Option<bool>,

    /// 消息类别过滤
    #[serde(rename = "category")]
    pub(crate) category: Option<String>,
}

/// 分页消息响应
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PaginatedMessages {
    pub(crate) data: Vec<MessageDto>,
    pub(crate) total: i64,
    pub(crate) page: i32,
    pub(crate) page_size: i32,
}

impl MessageDto {
    pub(crate) async fn get_notifications(
        pool: &sqlx::MySqlPool,
        username: &str,
        params: Option<&MessageQueryParams>,
    ) -> Result<PaginatedMessages, AppError> {
        use sqlx::{MySql, QueryBuilder};

        let default_params = MessageQueryParams::default();
        let params = params.unwrap_or(&default_params);
        let page = params.page.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(20).max(1).min(100); // 限制最大100条
        let offset = (page - 1) * page_size;

        // 构建查询条件 - 明确指定字段顺序，确保与 MessageDto 结构匹配
        let mut query_builder = QueryBuilder::<MySql>::new(
            "SELECT m.id, m.user_id, m.category, m.title, m.content, m.is_read, m.created_at 
             FROM messages m 
             INNER JOIN users u ON m.user_id = u.id 
             WHERE u.username = "
        );
        
        query_builder.push_bind(username);

        // 添加已读状态过滤
        if let Some(is_read) = params.is_read {
            query_builder.push(" AND m.is_read = ");
            query_builder.push_bind(is_read);
        }

        // 添加类别过滤
        if let Some(category) = &params.category {
            query_builder.push(" AND m.category = ");
            query_builder.push_bind(category);
        }

        // 获取总数
        let mut count_builder = QueryBuilder::<MySql>::new(
            "SELECT COUNT(*) as total FROM messages m 
             INNER JOIN users u ON m.user_id = u.id 
             WHERE u.username = "
        );
        
        count_builder.push_bind(username);

        // 添加已读状态过滤
        if let Some(is_read) = params.is_read {
            count_builder.push(" AND m.is_read = ");
            count_builder.push_bind(is_read);
        }

        // 添加类别过滤
        if let Some(category) = &params.category {
            count_builder.push(" AND m.category = ");
            count_builder.push_bind(category);
        }

        let total: i64 = count_builder
            .build_query_scalar::<i64>()
            .fetch_one(pool)
            .await
            .map_err(map_db_err!("Failed to count messages"))?;

        // 添加排序和分页
        query_builder.push(" ORDER BY m.created_at DESC");
        query_builder.push(" LIMIT ");
        query_builder.push_bind(page_size);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        // 执行查询
        let sql = query_builder.sql();
        tracing::debug!("Executing message query: {}", sql);
        tracing::debug!("Query params: username={}, page={}, page_size={}, is_read={:?}, category={:?}", 
            username, page, page_size, params.is_read, params.category);
        
        let messages = query_builder
            .build_query_as::<MessageDto>()
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch messages: {:?}", e);
                map_db_err!("Failed to fetch messages")(e)
            })?;

        tracing::debug!("Found {} messages, total: {}", messages.len(), total);

        Ok(PaginatedMessages {
            data: messages,
            total,
            page,
            page_size,
        })
    }

    pub(crate) async fn update_message_to_read(
        pool: &sqlx::MySqlPool,
        message_id: i32,
        username: &str,
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
