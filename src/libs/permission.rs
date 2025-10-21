use axum::http::Method;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::trace;

// Global permission trie storage
lazy_static! {
    pub static ref PERMISSION_TRIES: RwLock<HashMap<Method, PermissionTrie>> =
        RwLock::new(HashMap::new());
}

/// Configuration structure for permission rules
#[derive(Debug, Deserialize)]
pub struct PermissionConfig {
    pub rules: Vec<RuleConfig>,
}

/// Configuration for a single permission rule
#[derive(Debug, Deserialize)]
pub struct RuleConfig {
    pub path: String,
    pub method: String,
    pub required_permission: String,
    #[serde(default)]
    pub self_only: bool,
}

/// Runtime permission rule structure
#[derive(Debug, Clone)]
pub struct PermissionRule {
    pub method: Method,
    pub required_permission: String,
    pub self_only: bool,
    pub path_pattern: String,
}

/// Types of path segments in the radix trie
#[derive(Debug, Clone, PartialEq)]
enum SegmentType {
    /// Static path segment (exact match)
    Static,
    /// Contextual wildcard that maps to specific parameter names
    /// e.g., "username" or "tenant_hash"
    ContextualWildcard(String),
    /// Variable segment that captures path parameters
    /// e.g., {user_id}
    Variable(String),
}

/// Node in the radix trie for efficient path matching
#[derive(Debug, Clone)]
pub struct RadixNode {
    segment: String,
    segment_type: SegmentType,
    children: HashMap<String, RadixNode>,
    rule: Option<PermissionRule>,
}

/// Radix trie for efficient path-based permission matching
#[derive(Debug, Clone)]
pub struct PermissionTrie {
    root: RadixNode,
}

impl RadixNode {
    /// Creates a new RadixNode with the appropriate segment type
    /// based on the segment content and parent context
    fn new(segment: &str, parent_segment: Option<&str>) -> Self {
        let segment_type = if segment.starts_with('{') && segment.ends_with('}') {
            SegmentType::Variable(segment[1..segment.len() - 1].to_string())
        } else if segment == "*" {
            // Determine wildcard type based on parent segment context
            match parent_segment {
                Some("users") => SegmentType::ContextualWildcard("username".to_string()),
                Some("tenants") => SegmentType::ContextualWildcard("tenant_hash".to_string()),
                Some("markets") => SegmentType::ContextualWildcard("market_hash".to_string()),
                Some("providers") => SegmentType::ContextualWildcard("provider_hash".to_string()),
                _ => SegmentType::Variable("wildcard".to_string()),
            }
        } else {
            SegmentType::Static
        };

        RadixNode {
            segment: segment.to_string(),
            segment_type,
            children: HashMap::new(),
            rule: None,
        }
    }

    /// Returns the segment string
    pub fn get_segment(&self) -> &str {
        &self.segment
    }
}

impl Default for PermissionTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionTrie {
    /// Creates a new empty permission trie
    pub fn new() -> Self {
        Self {
            root: RadixNode::new("", None),
        }
    }

    /// Inserts a permission rule into the trie for the given path
    pub fn insert(&mut self, path: &str, rule: PermissionRule) {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        let mut current = &mut self.root;
        let mut parent_segment = None;

        for segment in segments {
            let node = RadixNode::new(segment, parent_segment);
            current = current.children.entry(segment.to_string()).or_insert(node);
            parent_segment = Some(segment);
        }
        current.rule = Some(rule);
    }

    /// Finds a matching permission rule for the given path
    /// Returns the rule and extracted path parameters
    pub fn find(&self, path: &str) -> Option<(Arc<PermissionRule>, HashMap<String, String>)> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        trace!("Finding path: {}, segments: {:?}", path, segments);
        self.find_recursive(&self.root, &segments, &mut HashMap::new())
    }

    /// Recursively searches for a matching rule in the trie
    fn find_recursive(
        &self,
        node: &RadixNode,
        remaining_segments: &[&str],
        params: &mut HashMap<String, String>,
    ) -> Option<(Arc<PermissionRule>, HashMap<String, String>)> {
        if remaining_segments.is_empty() {
            return node
                .rule
                .as_ref()
                .map(|rule| (Arc::new(rule.clone()), params.clone()));
        }

        let current_segment = remaining_segments[0];
        let remaining = &remaining_segments[1..];

        // 1. Try exact match first
        if let Some(child) = node.children.get(current_segment) {
            if let Some(result) = self.find_recursive(child, remaining, params) {
                return Some(result);
            }
        }

        // 2. Try wildcard and variable matching
        for child in node.children.values() {
            match &child.segment_type {
                SegmentType::ContextualWildcard(param_name) => {
                    params.insert(param_name.clone(), current_segment.to_string());
                    if let Some(result) = self.find_recursive(child, remaining, params) {
                        return Some(result);
                    }
                    params.remove(param_name);
                }
                SegmentType::Variable(var_name) => {
                    params.insert(var_name.clone(), current_segment.to_string());
                    if let Some(result) = self.find_recursive(child, remaining, params) {
                        return Some(result);
                    }
                    params.remove(var_name);
                }
                _ => continue,
            }
        }

        None
    }
}

/// Initializes permission tries from a YAML configuration file
pub async fn init_permissions(config_path: &str) -> Result<(), anyhow::Error> {
    let config_str = tokio::fs::read_to_string(config_path).await?;
    let config: PermissionConfig = serde_yaml::from_str(&config_str)?;

    let mut tries = HashMap::new();

    for rule in config.rules {
        let method = rule.method.parse::<Method>()?;
        let trie = tries
            .entry(method.clone())
            .or_insert_with(PermissionTrie::new);

        trie.insert(
            &rule.path,
            PermissionRule {
                method,
                required_permission: rule.required_permission,
                self_only: rule.self_only,
                path_pattern: rule.path.clone(),
            },
        );
    }

    let mut global_tries = PERMISSION_TRIES.write().await;
    *global_tries = tries;

    Ok(())
}

/// Verifies if user has all required permissions
/// Returns true if all required permissions are present in user_permissions
pub fn verify_permissions(required: &str, user_permissions: &[String]) -> bool {
    if required.trim().is_empty() {
        return true; // Empty permission requirement always passes
    }

    required.split(',').all(|perm| {
        let perm = perm.trim();
        !perm.is_empty() && user_permissions.contains(&perm.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contextual_wildcards() {
        let mut trie = PermissionTrie::new();

        // Add test route
        trie.insert(
            "/api/v1/users/*/tenants/*/products",
            PermissionRule {
                method: Method::GET,
                required_permission: "product:read".to_string(),
                self_only: true,
                path_pattern: "/api/v1/users/*/tenants/*/products".to_string(),
            },
        );

        // Test matching
        let (rule, params) = trie
            .find("/api/v1/users/john/tenants/company1/products")
            .unwrap();

        // Verify wildcards are correctly interpreted
        assert_eq!(params.get("username").unwrap(), "john");
        assert_eq!(params.get("tenant_hash").unwrap(), "company1");
        assert!(rule.self_only);
    }

    #[test]
    fn test_mixed_patterns() {
        let mut trie = PermissionTrie::new();

        trie.insert(
            "/api/v1/tenants/*/users/{user_id}/roles",
            PermissionRule {
                method: Method::POST,
                required_permission: "role:assign".to_string(),
                self_only: false,
                path_pattern: "/api/v1/tenants/*/users/{user_id}/roles".to_string(),
            },
        );

        let (rule, params) = trie
            .find("/api/v1/tenants/company1/users/u123/roles")
            .unwrap();

        assert_eq!(params.get("tenant_hash").unwrap(), "company1");
        assert_eq!(params.get("user_id").unwrap(), "u123");
        assert_eq!(rule.required_permission, "role:assign");
        assert!(!rule.self_only);
    }

    #[test]
    fn test_verify_permissions() {
        let user_permissions = vec!["user:read".to_string(), "tenant:admin".to_string()];

        // Single permission verification
        assert!(verify_permissions("user:read", &user_permissions));
        assert!(!verify_permissions("user:write", &user_permissions));

        // Multiple permission verification
        assert!(verify_permissions(
            "user:read,tenant:admin",
            &user_permissions
        ));
        assert!(!verify_permissions(
            "user:read,user:write",
            &user_permissions
        ));
    }

    #[test]
    fn test_deep_path_matching() {
        let mut trie = PermissionTrie::new();

        trie.insert(
            "/api/v1/tenants/*/users/*/addresses/*/phones/*",
            PermissionRule {
                method: Method::GET,
                required_permission: "user:read".to_string(),
                self_only: true,
                path_pattern: "/api/v1/tenants/*/users/*/addresses/*/phones/*".to_string(),
            },
        );

        let (rule, params) = trie
            .find("/api/v1/tenants/company1/users/john/addresses/home/phones/iphone")
            .unwrap();

        assert_eq!(params.get("tenant_hash").unwrap(), "company1");
        assert_eq!(params.get("username").unwrap(), "john");
        assert!(rule.self_only);
    }

    #[test]
    fn test_static_path_matching() {
        let mut trie = PermissionTrie::new();

        // Add static paths
        trie.insert(
            "/api/v1/health",
            PermissionRule {
                method: Method::GET,
                required_permission: "system:health".to_string(),
                self_only: false,
                path_pattern: "/api/v1/health".to_string(),
            },
        );

        trie.insert(
            "/api/v1/admin/settings",
            PermissionRule {
                method: Method::POST,
                required_permission: "admin:write".to_string(),
                self_only: false,
                path_pattern: "/api/v1/admin/settings".to_string(),
            },
        );

        // Test exact matching
        let (rule, params) = trie.find("/api/v1/health").unwrap();
        assert_eq!(rule.required_permission, "system:health");
        assert!(params.is_empty());

        let (rule, params) = trie.find("/api/v1/admin/settings").unwrap();
        assert_eq!(rule.required_permission, "admin:write");
        assert!(params.is_empty());

        // Test non-matching paths
        assert!(trie.find("/api/v1/health/status").is_none());
        assert!(trie.find("/api/v1/admin").is_none());
    }

    #[test]
    fn test_all_wildcard_types() {
        let mut trie = PermissionTrie::new();

        // Test all supported wildcard types
        trie.insert(
            "/api/v1/markets/*/data",
            PermissionRule {
                method: Method::GET,
                required_permission: "market:read".to_string(),
                self_only: false,
                path_pattern: "/api/v1/markets/*/data".to_string(),
            },
        );

        trie.insert(
            "/api/v1/providers/*/config",
            PermissionRule {
                method: Method::PUT,
                required_permission: "provider:config".to_string(),
                self_only: false,
                path_pattern: "/api/v1/providers/*/config".to_string(),
            },
        );

        // Test market_hash wildcard
        let (rule, params) = trie.find("/api/v1/markets/stock_market/data").unwrap();
        assert_eq!(params.get("market_hash").unwrap(), "stock_market");
        assert_eq!(rule.required_permission, "market:read");

        // Test provider_hash wildcard
        let (rule, params) = trie.find("/api/v1/providers/binance/config").unwrap();
        assert_eq!(params.get("provider_hash").unwrap(), "binance");
        assert_eq!(rule.required_permission, "provider:config");
    }

    #[test]
    fn test_unknown_wildcard_context() {
        let mut trie = PermissionTrie::new();

        // Test unknown context wildcard (should be treated as regular variable)
        trie.insert(
            "/api/v1/unknown/*/data",
            PermissionRule {
                method: Method::GET,
                required_permission: "data:read".to_string(),
                self_only: false,
                path_pattern: "/api/v1/unknown/*/data".to_string(),
            },
        );

        let (rule, params) = trie.find("/api/v1/unknown/test123/data").unwrap();
        assert_eq!(params.get("wildcard").unwrap(), "test123");
        assert_eq!(rule.required_permission, "data:read");
    }

    #[test]
    fn test_matching_priority() {
        let mut trie = PermissionTrie::new();

        // Add static path and wildcard path
        trie.insert(
            "/api/v1/users/admin",
            PermissionRule {
                method: Method::GET,
                required_permission: "admin:read".to_string(),
                self_only: false,
                path_pattern: "/api/v1/users/admin".to_string(),
            },
        );

        trie.insert(
            "/api/v1/users/*",
            PermissionRule {
                method: Method::GET,
                required_permission: "user:read".to_string(),
                self_only: true,
                path_pattern: "/api/v1/users/*".to_string(),
            },
        );

        // Exact match should take priority over wildcard match
        let (rule, params) = trie.find("/api/v1/users/admin").unwrap();
        assert_eq!(rule.required_permission, "admin:read");
        assert!(!rule.self_only);
        assert!(params.is_empty());

        // Other usernames should match wildcard
        let (rule, params) = trie.find("/api/v1/users/john").unwrap();
        assert_eq!(rule.required_permission, "user:read");
        assert!(rule.self_only);
        assert_eq!(params.get("username").unwrap(), "john");
    }

    #[test]
    fn test_edge_cases() {
        let mut trie = PermissionTrie::new();

        // Test root path
        trie.insert(
            "/",
            PermissionRule {
                method: Method::GET,
                required_permission: "root:access".to_string(),
                self_only: false,
                path_pattern: "/".to_string(),
            },
        );

        let (rule, params) = trie.find("/").unwrap();
        assert_eq!(rule.required_permission, "root:access");
        assert!(params.is_empty());

        // Test empty path (should be equivalent to root path)
        let (rule, params) = trie.find("").unwrap();
        assert_eq!(rule.required_permission, "root:access");
        assert!(params.is_empty());

        // Test non-existent paths
        assert!(trie.find("/nonexistent").is_none());
        assert!(trie.find("/api/v1/nonexistent").is_none());
    }

    #[test]
    fn test_path_with_trailing_slash() {
        let mut trie = PermissionTrie::new();

        trie.insert(
            "/api/v1/users",
            PermissionRule {
                method: Method::GET,
                required_permission: "user:list".to_string(),
                self_only: false,
                path_pattern: "/api/v1/users".to_string(),
            },
        );

        // Path with trailing slash should match
        let (rule, _) = trie.find("/api/v1/users/").unwrap();
        assert_eq!(rule.required_permission, "user:list");

        // Path without trailing slash should also match
        let (rule, _) = trie.find("/api/v1/users").unwrap();
        assert_eq!(rule.required_permission, "user:list");
    }

    #[test]
    fn test_complex_variable_patterns() {
        let mut trie = PermissionTrie::new();

        trie.insert(
            "/api/v1/{version}/users/{user_id}/posts/{post_id}",
            PermissionRule {
                method: Method::DELETE,
                required_permission: "post:delete".to_string(),
                self_only: true,
                path_pattern: "/api/v1/{version}/users/{user_id}/posts/{post_id}".to_string(),
            },
        );

        let (rule, params) = trie.find("/api/v1/v2/users/u123/posts/p456").unwrap();
        assert_eq!(rule.required_permission, "post:delete");
        assert_eq!(params.get("version").unwrap(), "v2");
        assert_eq!(params.get("user_id").unwrap(), "u123");
        assert_eq!(params.get("post_id").unwrap(), "p456");
        assert!(rule.self_only);
    }

    #[test]
    fn test_verify_permissions_edge_cases() {
        // Test empty permission list
        let empty_permissions: Vec<String> = vec![];
        assert!(!verify_permissions("user:read", &empty_permissions));

        // Test empty required permissions
        let user_permissions = vec!["user:read".to_string()];
        assert!(verify_permissions("", &user_permissions));
        assert!(verify_permissions("   ", &user_permissions)); // Only whitespace

        // Test permissions with whitespace
        let user_permissions = vec!["user:read".to_string(), "user:write".to_string()];
        assert!(verify_permissions(
            "user:read, user:write",
            &user_permissions
        ));
        assert!(verify_permissions(
            " user:read , user:write ",
            &user_permissions
        ));

        // Test single permission with whitespace
        let user_permissions = vec!["user:read".to_string()];
        assert!(verify_permissions(" user:read ", &user_permissions));

        // Test duplicate permissions
        assert!(verify_permissions("user:read,user:read", &user_permissions));

        // Test case with empty permissions in string
        let user_permissions = vec!["user:read".to_string()];
        assert!(!verify_permissions(
            "user:read,,user:write",
            &user_permissions
        )); // Contains empty permission should fail
    }

    #[test]
    fn test_radix_node_creation() {
        // Test static segment
        let node = RadixNode::new("api", None);
        assert_eq!(node.segment, "api");
        assert_eq!(node.segment_type, SegmentType::Static);

        // Test variable segment
        let node = RadixNode::new("{user_id}", None);
        assert_eq!(node.segment, "{user_id}");
        assert_eq!(
            node.segment_type,
            SegmentType::Variable("user_id".to_string())
        );

        // Test contextual wildcards
        let node = RadixNode::new("*", Some("users"));
        assert_eq!(node.segment, "*");
        assert_eq!(
            node.segment_type,
            SegmentType::ContextualWildcard("username".to_string())
        );

        let node = RadixNode::new("*", Some("tenants"));
        assert_eq!(
            node.segment_type,
            SegmentType::ContextualWildcard("tenant_hash".to_string())
        );

        let node = RadixNode::new("*", Some("markets"));
        assert_eq!(
            node.segment_type,
            SegmentType::ContextualWildcard("market_hash".to_string())
        );

        let node = RadixNode::new("*", Some("providers"));
        assert_eq!(
            node.segment_type,
            SegmentType::ContextualWildcard("provider_hash".to_string())
        );

        // Test unknown context wildcard
        let node = RadixNode::new("*", Some("unknown"));
        assert_eq!(
            node.segment_type,
            SegmentType::Variable("wildcard".to_string())
        );

        // Test get_segment method
        assert_eq!(node.get_segment(), "*");
    }

    #[test]
    fn test_method_parse_behavior() {
        use axum::http::Method;

        // Test valid methods
        assert!(Method::from_bytes(b"GET").is_ok());
        assert!(Method::from_bytes(b"POST").is_ok());
        assert!(Method::from_bytes(b"PUT").is_ok());
        assert!(Method::from_bytes(b"DELETE").is_ok());

        // Test invalid methods - empty string and containing spaces
        assert!(Method::from_bytes(b"").is_err());
        assert!(Method::from_bytes(b"GET POST").is_err()); // Contains space

        // Test string parsing
        assert!("GET".parse::<Method>().is_ok());
        assert!("".parse::<Method>().is_err()); // Empty string
        assert!("GET POST".parse::<Method>().is_err()); // Contains space
    }

    #[tokio::test]
    async fn test_init_permissions_success() {
        // Create temporary config file
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join("test_config.yaml");
        let config_content = r#"
rules:
  - path: "/api/v1/users/*"
    method: "GET"
    required_permission: "user:read"
    self_only: true
  - path: "/api/v1/tenants/*/users/{user_id}"
    method: "POST"
    required_permission: "user:create"
    self_only: false
"#;
        tokio::fs::write(&temp_file_path, config_content)
            .await
            .unwrap();

        // Test initialization
        let result = init_permissions(temp_file_path.to_str().unwrap()).await;
        assert!(result.is_ok());

        // Verify permission tries have been correctly initialized
        let tries = PERMISSION_TRIES.read().await;
        assert!(tries.contains_key(&Method::GET));
        assert!(tries.contains_key(&Method::POST));

        // Clean up temporary file
        let _ = tokio::fs::remove_file(&temp_file_path).await;
    }

    #[tokio::test]
    async fn test_init_permissions_file_not_found() {
        let result = init_permissions("/nonexistent/config.yaml").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_init_permissions_invalid_yaml() {
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join("invalid_config.yaml");
        let invalid_config = "invalid: yaml: content: [";
        tokio::fs::write(&temp_file_path, invalid_config)
            .await
            .unwrap();

        let result = init_permissions(temp_file_path.to_str().unwrap()).await;
        assert!(result.is_err());

        // Clean up temporary file
        let _ = tokio::fs::remove_file(&temp_file_path).await;
    }

    #[tokio::test]
    async fn test_init_permissions_invalid_method() {
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join("invalid_method_config.yaml");
        let config_content = r#"
rules:
  - path: "/api/v1/test"
    method: "GET POST"
    required_permission: "test:read"
"#;
        tokio::fs::write(&temp_file_path, config_content)
            .await
            .unwrap();

        let result = init_permissions(temp_file_path.to_str().unwrap()).await;
        assert!(result.is_err());

        // Clean up temporary file
        let _ = tokio::fs::remove_file(&temp_file_path).await;
    }

    #[test]
    fn test_multiple_wildcards_same_level() {
        let mut trie = PermissionTrie::new();

        // Add multiple wildcard paths at the same level
        trie.insert(
            "/api/v1/users/*/profile",
            PermissionRule {
                method: Method::GET,
                required_permission: "profile:read".to_string(),
                self_only: true,
                path_pattern: "/api/v1/users/*/profile".to_string(),
            },
        );

        trie.insert(
            "/api/v1/users/{user_id}/settings",
            PermissionRule {
                method: Method::GET,
                required_permission: "settings:read".to_string(),
                self_only: true,
                path_pattern: "/api/v1/users/{user_id}/settings".to_string(),
            },
        );

        // Test both patterns can match correctly
        let (rule, params) = trie.find("/api/v1/users/john/profile").unwrap();
        assert_eq!(rule.required_permission, "profile:read");
        assert_eq!(params.get("username").unwrap(), "john");

        let (rule, params) = trie.find("/api/v1/users/u123/settings").unwrap();
        assert_eq!(rule.required_permission, "settings:read");
        assert_eq!(params.get("user_id").unwrap(), "u123");
    }

    #[test]
    fn test_nested_variable_extraction() {
        let mut trie = PermissionTrie::new();

        trie.insert(
            "/api/v1/tenants/*/users/*/projects/{project_id}/tasks/{task_id}",
            PermissionRule {
                method: Method::PATCH,
                required_permission: "task:update".to_string(),
                self_only: false,
                path_pattern: "/api/v1/tenants/*/users/*/projects/{project_id}/tasks/{task_id}"
                    .to_string(),
            },
        );

        let (rule, params) = trie
            .find("/api/v1/tenants/company1/users/alice/projects/proj123/tasks/task456")
            .unwrap();

        assert_eq!(rule.required_permission, "task:update");
        assert_eq!(params.get("tenant_hash").unwrap(), "company1");
        assert_eq!(params.get("username").unwrap(), "alice");
        assert_eq!(params.get("project_id").unwrap(), "proj123");
        assert_eq!(params.get("task_id").unwrap(), "task456");
        assert!(!rule.self_only);
    }
}
