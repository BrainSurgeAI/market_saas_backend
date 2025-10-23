use axum::http::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::serializer_deserializer::{deserialize_method, serialize_method};

/// Runtime permission rule structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    #[serde(
        serialize_with = "serialize_method",
        deserialize_with = "deserialize_method"
    )]
    pub method: Method,
    pub required_permission: String,
    pub self_only: bool,
    pub path_pattern: String,
}

// 替换为新的定义
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RadixNode {
    /// 如果路径在此节点结束，则有关联的规则。
    /// 使用 Arc<T> 避免在 find 操作中进行深拷贝。
    pub rule: Option<Arc<PermissionRule>>,

    /// 存储静态路径的子节点，例如 "users", "posts"
    pub static_children: HashMap<String, RadixNode>,

    /// 存储变量路径的子节点，例如 "{user_id}"
    /// Box<T> 用于处理递归类型的大小问题。元组的第一个元素是变量名 ("user_id")。
    pub variable_child: Option<Box<(String, RadixNode)>>,

    /// 存储通配符路径的子节点，例如 "*filepath"
    /// 元组的第一个元素是通配符名 ("filepath")。
    pub wildcard_child: Option<Box<(String, RadixNode)>>,
}

impl RadixNode {
    pub fn new() -> Self {
        Self::default()
    }
}
