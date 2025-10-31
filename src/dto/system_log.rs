use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemLogDTO {
    pub event_time: Option<DateTime<Utc>>,
    pub log_level: String,
    pub category: String,
    pub user_id: Option<String>,
    pub tenant_id: Option<i32>,
    pub tenant_type: Option<String>,
    pub ip_address: Option<String>,
    pub request_id: Option<String>,
    pub component: String,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_stack: Option<String>,
    pub execution_time: Option<u32>,
}

impl SystemLogDTO {
    pub fn builder() -> SystemLogDTOBuilder {
        SystemLogDTOBuilder::new()
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.log_level = level.to_string();
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_error_code(mut self, error_code: &str) -> Self {
        self.error_code = Some(error_code.to_string());
        self
    }

    // pub fn with_error_stack(mut self, error_stack: &str) -> Self {
    //     self.error_stack = Some(error_stack.to_string());
    //     self
    // }

    pub fn with_execution_time(mut self, execution_time: u32) -> Self {
        self.execution_time = Some(execution_time);
        self
    }
}

pub struct SystemLogDTOBuilder {
    category: Option<String>,
    user_id: Option<String>,
    tenant_id: Option<i32>,
    tenant_type: Option<String>,
    ip_address: Option<String>,
    request_id: Option<String>,
    component: Option<String>,
    action: Option<String>,
    resource_type: Option<String>,
    resource_id: Option<String>,
}

impl Default for SystemLogDTOBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemLogDTOBuilder {
    pub fn new() -> Self {
        Self {
            category: None,
            user_id: None,
            tenant_id: None,
            tenant_type: None,
            ip_address: None,
            request_id: None,
            component: None,
            action: None,
            resource_type: None,
            resource_id: None,
        }
    }

    pub fn category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn tenant_id(mut self, tenant_id: i32) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn tenant_type(mut self, tenant_type: &str) -> Self {
        self.tenant_type = Some(tenant_type.to_string());
        self
    }

    pub fn request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }

    pub fn ip_address(mut self, ip_address: Option<&str>) -> Self {
        self.ip_address = ip_address.map(|s| s.to_string());
        self
    }

    pub fn component(mut self, component: &str) -> Self {
        self.component = Some(component.to_string());
        self
    }

    pub fn action(mut self, action: &str) -> Self {
        self.action = Some(action.to_string());
        self
    }

    pub fn resource_type(mut self, resource_type: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }

    pub fn resource_id(mut self, resource_id: &str) -> Self {
        self.resource_id = Some(resource_id.to_string());
        self
    }

    pub fn build(self) -> SystemLogDTO {
        SystemLogDTO {
            event_time: None, // 将在插入时设置
            log_level: LogLevel::Info.to_string(),
            category: self.category.unwrap_or_else(|| "system".to_string()),
            user_id: self.user_id,
            tenant_id: self.tenant_id,
            tenant_type: self.tenant_type,
            ip_address: self.ip_address,
            request_id: self.request_id,
            component: self.component.unwrap_or_else(|| "unknown".to_string()),
            action: self.action.unwrap_or_else(|| "unknown".to_string()),
            resource_type: self.resource_type,
            resource_id: self.resource_id,
            message: "".to_string(), // 将通过with_message方法设置
            details: None,
            error_code: None,
            error_stack: None,
            execution_time: None,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serde_json::json;

//     #[test]
//     fn test_system_log_dto_builder() {
//         let log = SystemLogDTO::builder()
//             .category("test")
//             .user_id("user1")
//             .tenant_id(123)
//             .component("test_component")
//             .action("create")
//             .build()
//             .with_level(LogLevel::Info)
//             .with_message("测试日志")
//             .with_execution_time(42);

//         assert_eq!(log.category, "test");
//         assert_eq!(log.user_id, Some("user1".to_string()));
//         assert_eq!(log.tenant_id, Some(123));
//         assert_eq!(log.component, "test_component");
//         assert_eq!(log.action, "create");
//         assert_eq!(log.log_level, "INFO");
//         assert_eq!(log.message, "测试日志");
//         assert_eq!(log.execution_time, Some(42));
//     }

//     #[test]
//     fn test_with_details() {
//         let log = SystemLogDTO::builder().build().with_details(json!({
//             "key1": "value1",
//             "key2": 42
//         }));

//         if let Some(details) = log.details {
//             assert_eq!(details["key1"], "value1");
//             assert_eq!(details["key2"], 42);
//         } else {
//             panic!("Details should be set");
//         }
//     }

//     #[test]
//     fn test_with_error_info() {
//         let log = SystemLogDTO::builder()
//             .build()
//             .with_level(LogLevel::Error)
//             .with_error_code("ERR_001")
//             .with_error_stack("Error stack trace");

//         assert_eq!(log.log_level, "ERROR");
//         assert_eq!(log.error_code, Some("ERR_001".to_string()));
//         assert_eq!(log.error_stack, Some("Error stack trace".to_string()));
//     }
// }
