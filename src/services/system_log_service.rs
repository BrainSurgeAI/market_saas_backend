use std::sync::Arc;
// use std::time::Instant;

use crate::{
    common::AppError,
    dto::system_log::SystemLogDTO,
    repositories::system_log_repo::SystemLogRepository,
};

pub(crate) struct SystemLogService<T> {
    repo: Arc<T>,
}

impl<T> SystemLogService<T>
where
    T: SystemLogRepository + Send + Sync,
{
    pub(crate) fn new(repo: Arc<T>) -> Self {
        Self { repo }
    }

    pub(crate) async fn log_info(&self, log: SystemLogDTO) -> Result<(), AppError> {
        self.repo.log_info(log).await
    }

    pub(crate) async fn log_error(&self, log: SystemLogDTO) -> Result<(), AppError> {
        self.repo.log_error(log).await
    }

    // pub async fn log_warn(&self, log: SystemLogDTO) -> Result<(), AppError> {
    //     self.repo.log_warn(log).await
    // }

    // pub async fn log_debug(&self, log: SystemLogDTO) -> Result<(), AppError> {
    //     self.repo.log_debug(log).await
    // }

    // pub async fn log_fatal(&self, log: SystemLogDTO) -> Result<(), AppError> {
    //     self.repo.log_fatal(log).await
    // }

    // 用于业务操作的便捷方法
    // async fn log_operation(
    //     &self,
    //     category: &str,
    //     component: &str,
    //     action: &str,
    //     resource_type: Option<&str>,
    //     resource_id: Option<&str>,
    //     user_id: Option<&str>,
    //     tenant_id: Option<i32>,
    //     tenant_type: Option<&str>,
    //     message: &str,
    //     details: Option<serde_json::Value>,
    // ) -> Result<(), AppError> {
    //     let mut log_builder = SystemLogDTO::builder()
    //         .category(category)
    //         .component(component)
    //         .action(action);

    //     if let Some(rt) = resource_type {
    //         log_builder = log_builder.resource_type(rt);
    //     }

    //     if let Some(rid) = resource_id {
    //         log_builder = log_builder.resource_id(rid);
    //     }

    //     if let Some(uid) = user_id {
    //         log_builder = log_builder.user_id(uid);
    //     }

    //     if let Some(tid) = tenant_id {
    //         log_builder = log_builder.tenant_id(tid);
    //     }

    //     if let Some(tt) = tenant_type {
    //         log_builder = log_builder.tenant_type(tt);
    //     }

    //     let log = log_builder.build().with_message(message);

    //     let log = if let Some(details_value) = details {
    //         log.with_details(details_value)
    //     } else {
    //         log
    //     };

    //     self.log_info(log).await
    // }

    // 计时器上下文，用于记录执行时间
    // fn timer(&self) -> OperationTimer {
    //     OperationTimer {
    //         start: Instant::now(),
    //     }
    // }

    // 记录异常操作
    // async fn log_exception(
    //     &self,
    //     category: &str,
    //     component: &str,
    //     action: &str,
    //     error: &AppError,
    //     details: Option<serde_json::Value>,
    //     user_id: Option<&str>,
    //     tenant_id: Option<i32>,
    //     tenant_type: Option<&str>,
    // ) -> Result<(), AppError> {
    //     let mut log_builder = SystemLogDTO::builder()
    //         .category(category)
    //         .component(component)
    //         .action(action);

    //     if let Some(uid) = user_id {
    //         log_builder = log_builder.user_id(uid);
    //     }

    //     if let Some(tid) = tenant_id {
    //         log_builder = log_builder.tenant_id(tid);
    //     }

    //     if let Some(tt) = tenant_type {
    //         log_builder = log_builder.tenant_type(tt);
    //     }

    //     let error_message = error.to_string();
    //     let error_code = error_to_code(error);

    //     let log = log_builder
    //         .build()
    //         .with_level(LogLevel::Error)
    //         .with_message(&format!("操作失败: {}", error_message))
    //         .with_error_code(&error_code);
    //     //    .with_error_stack(&error_message);

    //     let log = if let Some(details_value) = details {
    //         log.with_details(details_value)
    //     } else {
    //         log
    //     };

    //     self.repo.log_error(log).await
    // }
}

// struct OperationTimer {
//     start: Instant,
// }

// impl OperationTimer {
//     fn elapsed_ms(&self) -> u32 {
//         self.start.elapsed().as_millis() as u32
//     }
// }

// 将AppError转换为错误代码
// fn error_to_code(error: &AppError) -> String {
//     match error {
//         AppError::NotFound(_) => "NOT_FOUND".to_string(),
//         AppError::Forbidden(_) => "FORBIDDEN".to_string(),
//         AppError::Conflict(_) => "CONFLICT".to_string(),
//         AppError::Validation(_) => "VALIDATION".to_string(),
//         AppError::Internal(_) => "INTERNAL_ERROR".to_string(),
//         AppError::Database(_) => "DATABASE_ERROR".to_string(),
//         _ => "UNKNOWN_ERROR".to_string(),
//     }
// }



// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::repositories::system_log_repo::tests::MockSystemLogRepo;
//     use mockall::predicate::*;
//     use serde_json::json;

//     #[tokio::test]
//     async fn test_log_operation() {
//         let mut mock = MockSystemLogRepo::new();

//         mock.expect_log_info()
//             .with(function(|log: &SystemLogDTO| {
//                 log.log_level == "INFO"
//                     && log.category == "test"
//                     && log.component == "test_component"
//                     && log.action == "create"
//                     && log.message == "Test operation"
//             }))
//             .times(1)
//             .returning(|_| Ok(()));

//         let service = SystemLogService::new(Arc::new(mock));

//         let result = service
//             .log_operation(
//                 "test",
//                 "test_component",
//                 "create",
//                 Some("user"),
//                 Some("123"),
//                 Some("admin"),
//                 Some(1),
//                 Some("ADMIN"),
//                 "Test operation",
//                 Some(json!({"test": "data"})),
//             )
//             .await;

//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_log_exception() {
//         let mut mock = MockSystemLogRepo::new();

//         mock.expect_log_error()
//             .with(function(|log: &SystemLogDTO| {
//                 log.log_level == "ERROR"
//                     && log.category == "test"
//                     && log.error_code == Some("NOT_FOUND".to_string())
//             }))
//             .times(1)
//             .returning(|_| Ok(()));

//         let service = SystemLogService::new(Arc::new(mock));
//         let error = AppError::NotFound("Resource not found".to_string());

//         let result = service
//             .log_exception(
//                 "test",
//                 "test_component",
//                 "find",
//                 &error,
//                 Some(json!({"resource_id": "123"})),
//                 Some("admin"),
//                 Some(1),
//                 Some("ADMIN"),
//             )
//             .await;

//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_timer() {
//         let mock = MockSystemLogRepo::new();
//         let service = SystemLogService::new(Arc::new(mock));

//         let timer = service.timer();
//         // 等待一段时间
//         std::thread::sleep(std::time::Duration::from_millis(10));

//         let elapsed = timer.elapsed_ms();
//         assert!(elapsed >= 10);
//     }
// }
