use crate::{
    common::AppError,
    dto::system_log::{LogLevel, SystemLogDTO},
};
use async_trait::async_trait;
use sqlx::{MySql, Pool};
use std::sync::Arc;

#[allow(dead_code)]
#[async_trait]
pub trait SystemLogRepository: Send + Sync {
    async fn log_info(&self, log: SystemLogDTO) -> Result<(), AppError>;
    async fn log_error(&self, log: SystemLogDTO) -> Result<(), AppError>;
    async fn log_warn(&self, log: SystemLogDTO) -> Result<(), AppError>;
    async fn log_debug(&self, log: SystemLogDTO) -> Result<(), AppError>;
    async fn log_fatal(&self, log: SystemLogDTO) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct MySqlSystemLogRepository {
    pool: Arc<Pool<MySql>>,
}

impl MySqlSystemLogRepository {
    pub fn new(pool: Arc<Pool<MySql>>) -> Self {
        Self { pool }
    }

    async fn insert_log(&self, log: SystemLogDTO) -> Result<(), AppError> {
        let query = r#"
        INSERT INTO system_log (
            log_level, category, user_id, tenant_id, tenant_type,
            ip_address, request_id, component, action, resource_type,
            resource_id, message, details, error_code, error_stack, execution_time
        ) VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        )
        "#;

        sqlx::query(query)
            .bind(&log.log_level)
            .bind(&log.category)
            .bind(&log.user_id)
            .bind(log.tenant_id)
            .bind(&log.tenant_type)
            .bind(&log.ip_address)
            .bind(&log.request_id)
            .bind(&log.component)
            .bind(&log.action)
            .bind(&log.resource_type)
            .bind(&log.resource_id)
            .bind(&log.message)
            .bind(serde_json::to_string(&log.details).ok())
            .bind(&log.error_code)
            .bind(&log.error_stack)
            .bind(log.execution_time)
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                // 日志写入失败不应该影响正常业务，所以只打印错误，不返回
                tracing::error!("Failed to insert system log: {}", e);
                AppError::Internal(format!("Failed to insert system log: {}", e))
            })?;

        Ok(())
    }
}

#[async_trait]
impl SystemLogRepository for MySqlSystemLogRepository {
    async fn log_info(&self, log: SystemLogDTO) -> Result<(), AppError> {
        let log = log.with_level(LogLevel::Info);
        self.insert_log(log).await
    }

    async fn log_error(&self, log: SystemLogDTO) -> Result<(), AppError> {
        let log = log.with_level(LogLevel::Error);
        self.insert_log(log).await
    }

    async fn log_warn(&self, log: SystemLogDTO) -> Result<(), AppError> {
        let log = log.with_level(LogLevel::Warn);
        self.insert_log(log).await
    }

    async fn log_debug(&self, log: SystemLogDTO) -> Result<(), AppError> {
        let log = log.with_level(LogLevel::Debug);
        self.insert_log(log).await
    }

    async fn log_fatal(&self, log: SystemLogDTO) -> Result<(), AppError> {
        let log = log.with_level(LogLevel::Fatal);
        self.insert_log(log).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        pub SystemLogRepo {}
        #[async_trait]
        impl SystemLogRepository for SystemLogRepo {
            async fn log_info(&self, log: SystemLogDTO) -> Result<(), AppError>;
            async fn log_error(&self, log: SystemLogDTO) -> Result<(), AppError>;
            async fn log_warn(&self, log: SystemLogDTO) -> Result<(), AppError>;
            async fn log_debug(&self, log: SystemLogDTO) -> Result<(), AppError>;
            async fn log_fatal(&self, log: SystemLogDTO) -> Result<(), AppError>;
        }
    }

    #[tokio::test]
    async fn test_log_info() {
        let mut mock = MockSystemLogRepo::new();

        mock.expect_log_info()
            .with(function(|log: &SystemLogDTO| {
                log.log_level == "INFO" && log.message == "Test message"
            }))
            .times(1)
            .returning(|_| Ok(()));

        let log = SystemLogDTO::builder()
            .category("test")
            .component("test_component")
            .action("create")
            .build()
            .with_message("Test message");

        let result = mock.log_info(log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_log_error() {
        let mut mock = MockSystemLogRepo::new();

        mock.expect_log_error()
            .with(function(|log: &SystemLogDTO| {
                log.message == "Error message"
                    && log.error_code == Some("ERR_001".to_string())
                    && log.category == "test"
                    && log.component == "test_component"
                    && log.action == "create"
            }))
            .times(1)
            .returning(|_| Ok(()));

        let log = SystemLogDTO::builder()
            .category("test")
            .component("test_component")
            .action("create")
            .build()
            .with_message("Error message")
            .with_error_code("ERR_001");

        let result = mock.log_error(log).await;
        assert!(result.is_ok());
    }
}
