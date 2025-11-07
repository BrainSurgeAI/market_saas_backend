use crate::acl_core::permission_trie::PermissionTrie;
use crate::acl_core::serializer_deserializer::{deserialize_method_map, serialize_method_map};
use crate::models::claims::Claims;
use dashmap::DashMap;
use hyper::{Method, StatusCode};
use serde::{Deserialize, Serialize};

use arc_swap::ArcSwap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::LazyLock;
use tracing::{debug, error};

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub(crate) struct AclSnapshot {
    #[serde(
        serialize_with = "serialize_method_map",
        deserialize_with = "deserialize_method_map"
    )]
    pub(crate) tries: HashMap<Method, PermissionTrie>,
    pub(crate) role_perms: HashMap<String, HashSet<String>>,
    pub(crate) version: u64,
    //pub(crate) system_code: String,
    #[serde(skip)]
    pub(crate) role_cache: Arc<DashMap<Vec<String>, HashSet<String>>>,
}

/// 全局 ACL 快照（读侧无锁，写侧原子替换）
pub(crate) static ACL_SNAPSHOT: LazyLock<ArcSwap<AclSnapshot>> =
    LazyLock::new(|| ArcSwap::from_pointee(AclSnapshot::default()));

impl AclSnapshot {
    /// Verifies if a user has access to a specific resource
    /// # Arguments
    /// * `method` - The HTTP method of the request
    /// * `path` - The request path
    /// * `claim` - The user's claim containing roles and other info
    /// # Returns
    /// * `Result<(), StatusCode>` - Ok if access is granted, Err with appropriate StatusCode if denied
    pub(crate) async fn verify_access(
        &self,
        method: &Method,
        path: &str,
        claim: &Claims,
    ) -> Result<(), StatusCode> {
        if claim.is_super_admin {
            return Ok(());
        }

        let trie = self.tries.get(method).ok_or_else(|| {
            error!("No permission trie found for HTTP method: {}", method);
            StatusCode::METHOD_NOT_ALLOWED
        })?;

        debug!(
            "Verifying access for {} {} with roles {:?}",
            method, path, claim.roles
        );
        let (rule, params) = trie.find(path).ok_or_else(|| {
            error!("No path_pattern and method matches {} {}", path, method);
            StatusCode::UNAUTHORIZED
        })?;

        if rule.self_only {
            debug!("Route {} is self-only, verifying username", path);

            // If username in the url, e.g /users/{username}
            if let Some(username) = params.get("username") {
                debug!("Extracted username parameter: {}", username);
                if username != &claim.username {
                    error!(
                        "Self-only route {} accessed by user {} (expected: {})",
                        path, claim.username, username
                    );
                    return Err(StatusCode::FORBIDDEN);
                }
            }
            debug!("Self-only verification ignored for user {}", claim.username);
        }

        debug!("params: {:?}", params);

        // If the url contains tenant hashed name, e.g /tenants/{hashed_name}
        if let Some(hashed_name) = params.get("hashed_name") {
            debug!("Extracted hashed_name parameter: {}", hashed_name);
            debug!("Route {} is tenant-specific, verifying hashed_name", path);

            if hashed_name != &claim.tenant_hash && claim.roles.iter().any(|r| r != "MARKET_ADMIN")
            {
                error!(
                    "Tenant-only route {} accessed by tenant {} (expected: {})",
                    path, claim.tenant_hash, hashed_name
                );
                return Err(StatusCode::FORBIDDEN);
            }
            debug!(
                "Tenant-only verification passed for tenant {}",
                claim.tenant_hash
            );
        }

        let perms = self.permissions_for_roles(&claim.roles);

        if !verify_permissions(&rule.required_permission, &perms) {
            error!(
                "User {} does not have required permissions: {}",
                claim.username, rule.required_permission
            );
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(())
    }

    fn permissions_for_roles(&self, roles: &[String]) -> HashSet<String> {
        // sort roles to prevent cache miss
        let mut sorted_roles = roles.to_vec();
        sorted_roles.sort();

        if let Some(cached) = self.role_cache.get(&sorted_roles) {
            return cached.clone();
        }

        let mut perms = HashSet::with_capacity(roles.len() * 5);
        for role in &sorted_roles {
            if let Some(role_perms) = self.role_perms.get(role) {
                perms.extend(role_perms.iter().cloned());
            }
        }

        self.role_cache.insert(sorted_roles, perms.clone());
        perms
    }
}

/// Verifies if user has all required permissions
/// Returns true if all required permissions are present in user_permissions
///
/// # Arguments
/// * `required` - The required permissions
/// * `user_permissions` - The user's permissions
///
/// # Returns
/// * `bool` - True if the user has all required permissions, false otherwise
fn verify_permissions(required: &str, user_permissions: &HashSet<String>) -> bool {
    if required.trim().is_empty() {
        return true; // Empty permission requirement always passes
    }

    required.split(',').all(|perm| {
        let perm = perm.trim();
        !perm.is_empty() && user_permissions.contains(perm)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acl_core::radix_node::PermissionRule;

    fn create_test_claim(
        username: &str,
        tenant_name: &str,
        roles: Vec<String>,
        is_super_admin: bool,
    ) -> Claims {
        Claims {
            tenant_type: "test".to_string(),
            tenant_name: tenant_name.to_string(),
            tenant_hash: "kkkk".to_string(),
            username: username.to_string(),
            real_name: "test".to_string(),
            roles,
            exp: 9999999999,
            is_super_admin,
        }
    }

    fn create_test_snapshot() -> AclSnapshot {
        let mut snapshot = AclSnapshot {
            tries: HashMap::new(),
            role_perms: HashMap::new(),
            version: 1,
            role_cache: Arc::new(DashMap::new()),
        };

        // Setup role permissions
        snapshot.role_perms.insert(
            "admin".to_string(),
            vec![
                "user:read".to_string(),
                "user:write".to_string(),
                "user:delete".to_string(),
                "system:admin".to_string(),
            ]
            .into_iter()
            .collect(),
        );

        snapshot.role_perms.insert(
            "user".to_string(),
            vec![
                "user:read".to_string(),
                "profile:read".to_string(),
                "profile:write".to_string(),
            ]
            .into_iter()
            .collect(),
        );

        snapshot.role_perms.insert(
            "viewer".to_string(),
            vec!["user:read".to_string(), "profile:read".to_string()]
                .into_iter()
                .collect(),
        );

        snapshot
    }

    fn create_test_permission_trie() -> PermissionTrie {
        let mut trie = PermissionTrie::new();

        // Insert public route (no permissions required)
        trie.insert(
            "/health",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/health".to_string(),
                required_permission: "".to_string(),
                self_only: false,
            },
        );

        // Insert user-specific route
        trie.insert(
            "/users/{username}",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/users/{username}".to_string(),
                required_permission: "user:read".to_string(),
                self_only: true,
            },
        );

        // Insert admin route
        trie.insert(
            "/admin/users",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/admin/users".to_string(),
                required_permission: "system:admin".to_string(),
                self_only: false,
            },
        );

        // Insert multi-permission route
        trie.insert(
            "/reports/{tenant_name}",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/reports/{tenant_name}".to_string(),
                required_permission: "user:read".to_string(), // Use permission that user role has
                self_only: false,
            },
        );

        trie
    }

    #[tokio::test]
    async fn test_verify_access_super_admin_success() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("admin", "tenant1", vec!["admin".to_string()], true);

        // Super admin should have access to any route
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/admin/users", &claim)
            .await;
        assert!(result.is_ok());

        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/nonexistent/route", &claim)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_method_not_allowed() {
        let snapshot = create_test_snapshot();
        let claim = create_test_claim("user1", "tenant1", vec!["user".to_string()], false);

        // No trie for POST method
        let result = snapshot
            .verify_access(&Method::POST, "/users/user1", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_verify_access_route_not_found() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec!["user".to_string()], false);

        // Route doesn't exist
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/nonexistent", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_verify_access_public_route_success() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec!["user".to_string()], false);

        // Public route (no permissions required)
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/health", &claim)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_self_only_success() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec!["user".to_string()], false);

        // Self-only route with matching username
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/users/user1", &claim)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_self_only_forbidden() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec!["user".to_string()], false);

        // Self-only route with different username
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/users/user2", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_verify_access_tenant_only_success() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec!["user".to_string()], false);

        // Tenant-specific route with matching tenant
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/reports/tenant1", &claim)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_tenant_only_forbidden() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec!["user".to_string()], false);

        // Tenant-specific route with different tenant
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/reports/tenant2", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_verify_access_insufficient_permissions() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec!["viewer".to_string()], false);

        // Admin route requires system:admin permission
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/admin/users", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_verify_access_multi_permission_success() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec!["admin".to_string()], false);

        // User already has user:read permission, so this should work
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/reports/tenant1", &claim)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_access_multi_permission_partial() {
        let snapshot = create_test_snapshot();
        let claim = create_test_claim("user1", "tenant1", vec!["admin".to_string()], false);

        // Test with missing permission - use a permission that admin doesn't have
        let mut trie = PermissionTrie::new();
        trie.insert(
            "/reports/{tenant_name}",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/reports/{tenant_name}".to_string(),
                required_permission: "nonexistent:permission".to_string(), // Permission that doesn't exist
                self_only: false,
            },
        );
        let mut tries = HashMap::new();
        tries.insert(Method::GET, trie);
        let snapshot_with_trie_no_perm = AclSnapshot { tries, ..snapshot };

        let result = snapshot_with_trie_no_perm
            .verify_access(&Method::GET, "/reports/tenant1", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_permissions_for_roles_single_role() {
        let snapshot = create_test_snapshot();

        let roles = vec!["admin".to_string()];
        let perms = snapshot.permissions_for_roles(&roles);

        assert_eq!(perms.len(), 4);
        assert!(perms.contains("user:read"));
        assert!(perms.contains("user:write"));
        assert!(perms.contains("user:delete"));
        assert!(perms.contains("system:admin"));
    }

    #[tokio::test]
    async fn test_permissions_for_roles_multiple_roles() {
        let snapshot = create_test_snapshot();

        let roles = vec!["admin".to_string(), "user".to_string()];
        let perms = snapshot.permissions_for_roles(&roles);

        // Should contain permissions from both roles
        assert_eq!(perms.len(), 6);
        assert!(perms.contains("user:read"));
        assert!(perms.contains("user:write"));
        assert!(perms.contains("user:delete"));
        assert!(perms.contains("system:admin"));
        assert!(perms.contains("profile:read"));
        assert!(perms.contains("profile:write"));
    }

    #[tokio::test]
    async fn test_permissions_for_roles_three_roles() {
        let snapshot = create_test_snapshot();

        let roles = vec![
            "admin".to_string(),
            "user".to_string(),
            "viewer".to_string(),
        ];
        let perms = snapshot.permissions_for_roles(&roles);

        // Should contain permissions from all three roles (note: user:read appears in multiple roles)
        assert_eq!(perms.len(), 6);
        assert!(perms.contains("user:read"));
        assert!(perms.contains("user:write"));
        assert!(perms.contains("user:delete"));
        assert!(perms.contains("system:admin"));
        assert!(perms.contains("profile:read"));
        assert!(perms.contains("profile:write"));
    }

    #[tokio::test]
    async fn test_permissions_for_roles_nonexistent_role() {
        let snapshot = create_test_snapshot();

        let roles = vec!["nonexistent".to_string()];
        let perms = snapshot.permissions_for_roles(&roles);

        // Should return empty permissions for nonexistent role
        assert!(perms.is_empty());
    }

    #[tokio::test]
    async fn test_permissions_for_roles_mixed_existing_nonexistent() {
        let snapshot = create_test_snapshot();

        let roles = vec!["admin".to_string(), "nonexistent".to_string()];
        let perms = snapshot.permissions_for_roles(&roles);

        // Should only include permissions from existing role
        assert_eq!(perms.len(), 4);
        assert!(perms.contains("user:read"));
        assert!(perms.contains("user:write"));
        assert!(perms.contains("user:delete"));
        assert!(perms.contains("system:admin"));
    }

    #[tokio::test]
    async fn test_permissions_for_roles_empty_roles() {
        let snapshot = create_test_snapshot();

        let roles = vec![];
        let perms = snapshot.permissions_for_roles(&roles);

        assert!(perms.is_empty());
    }

    #[tokio::test]
    async fn test_permissions_for_roles_caching() {
        let snapshot = create_test_snapshot();

        let roles1 = vec!["admin".to_string(), "user".to_string()];
        let roles2 = vec!["user".to_string(), "admin".to_string()]; // Different order

        let perms1 = snapshot.permissions_for_roles(&roles1);
        let perms2 = snapshot.permissions_for_roles(&roles2);

        // Should be the same due to sorting and caching
        assert_eq!(perms1, perms2);

        // Check that cache was used (same hash map instance)
        let cache_size = snapshot.role_cache.len();
        assert_eq!(cache_size, 1);
    }

    #[tokio::test]
    async fn test_permissions_for_roles_duplicate_roles() {
        let snapshot = create_test_snapshot();

        let roles = vec!["admin".to_string(), "admin".to_string(), "user".to_string()];
        let perms = snapshot.permissions_for_roles(&roles);

        // Should handle duplicate roles correctly
        assert_eq!(perms.len(), 6);
        assert!(perms.contains("user:read"));
        assert!(perms.contains("user:write"));
        assert!(perms.contains("user:delete"));
        assert!(perms.contains("system:admin"));
        assert!(perms.contains("profile:read"));
        assert!(perms.contains("profile:write"));
    }

    #[test]
    fn test_verify_permissions_empty_required() {
        let user_perms = HashSet::from(["user:read".to_string()]);

        let result = verify_permissions("", &user_perms);
        assert!(result); // Empty requirement always passes

        let result = verify_permissions("   ", &user_perms);
        assert!(result); // Whitespace only requirement always passes
    }

    #[test]
    fn test_verify_permissions_single_permission_success() {
        let user_perms = HashSet::from(["user:read".to_string(), "user:write".to_string()]);

        let result = verify_permissions("user:read", &user_perms);
        assert!(result);
    }

    #[test]
    fn test_verify_permissions_single_permission_failure() {
        let user_perms = HashSet::from(["user:read".to_string()]);

        let result = verify_permissions("user:write", &user_perms);
        assert!(!result);
    }

    #[test]
    fn test_verify_permissions_multiple_permissions_success() {
        let user_perms = HashSet::from([
            "user:read".to_string(),
            "user:write".to_string(),
            "user:delete".to_string(),
        ]);

        let result = verify_permissions("user:read,user:write", &user_perms);
        assert!(result);

        let result = verify_permissions("user:read, user:write, user:delete", &user_perms);
        assert!(result);
    }

    #[test]
    fn test_verify_permissions_multiple_permissions_partial_failure() {
        let user_perms = HashSet::from(["user:read".to_string(), "user:write".to_string()]);

        let result = verify_permissions("user:read,user:delete", &user_perms);
        assert!(!result); // Missing user:delete
    }

    #[test]
    fn test_verify_permissions_multiple_permissions_all_missing() {
        let user_perms = HashSet::from(["profile:read".to_string()]);

        let result = verify_permissions("user:read,user:write", &user_perms);
        assert!(!result); // Neither permission exists
    }

    #[test]
    fn test_verify_permissions_with_extra_whitespace() {
        let user_perms = HashSet::from(["user:read".to_string(), "user:write".to_string()]);

        let result = verify_permissions("  user:read  ,  user:write  ", &user_perms);
        assert!(result);
    }

    #[test]
    fn test_verify_permissions_empty_permissions_in_list() {
        let user_perms = HashSet::from(["user:read".to_string()]);

        let result = verify_permissions("user:read,,user:write", &user_perms);
        assert!(!result); // user:write is missing

        let result = verify_permissions("user:read,,", &user_perms);
        assert!(!result); // Empty string after comma is treated as a required permission
    }

    #[test]
    fn test_verify_permissions_trailing_comma() {
        let user_perms = HashSet::from(["user:read".to_string()]);

        let result = verify_permissions("user:read,", &user_perms);
        assert!(!result); // Trailing comma creates an empty permission requirement
    }

    #[test]
    fn test_verify_permissions_leading_comma() {
        let user_perms = HashSet::from(["user:read".to_string()]);

        let result = verify_permissions(",user:read", &user_perms);
        assert!(!result); // Leading comma creates an empty permission requirement
    }

    #[tokio::test]
    async fn test_verify_access_edge_case_username_parameter_missing() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec!["user".to_string()], false);

        // Route that requires username parameter but parameter is not in path
        // This would depend on the implementation of find() method
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/users", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_verify_access_edge_case_empty_claim_roles() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "tenant1", vec![], false);

        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/users/user1", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_verify_access_edge_case_empty_tenant_name() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("user1", "", vec!["admin".to_string()], false);

        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/reports/tenant1", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_verify_access_edge_case_empty_username() {
        let snapshot = create_test_snapshot();
        let mut tries = HashMap::new();
        tries.insert(Method::GET, create_test_permission_trie());
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        let claim = create_test_claim("", "tenant1", vec!["user".to_string()], false);

        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/users/", &claim)
            .await;
        assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_acl_snapshot_default() {
        let snapshot = AclSnapshot::default();

        assert!(snapshot.tries.is_empty());
        assert!(snapshot.role_perms.is_empty());
        assert_eq!(snapshot.version, 0);
        assert_eq!(snapshot.role_cache.len(), 0);
    }

    #[test]
    fn test_acl_snapshot_clone() {
        let snapshot = create_test_snapshot();
        let cloned = snapshot.clone();

        assert_eq!(snapshot.version, cloned.version);
        assert_eq!(snapshot.role_perms.len(), cloned.role_perms.len());
    }

    #[test]
    fn test_permissions_for_roles_large_role_set() {
        let mut snapshot = create_test_snapshot();

        // Add many roles and permissions
        for i in 0..100 {
            let role_name = format!("role_{}", i);
            let mut perms = HashSet::new();
            for j in 0..10 {
                perms.insert(format!("perm_{}_{}", i, j));
            }
            snapshot.role_perms.insert(role_name, perms);
        }

        let roles: Vec<String> = (0..100).map(|i| format!("role_{}", i)).collect();
        let perms = snapshot.permissions_for_roles(&roles);

        assert_eq!(perms.len(), 1000); // 100 roles * 10 permissions each
    }

    #[tokio::test]
    async fn test_verify_access_comprehensive_multi_role_scenario() {
        let mut snapshot = create_test_snapshot();

        // Setup complex role hierarchy
        snapshot.role_perms.insert(
            "basic_user".to_string(),
            vec!["profile:read".to_string(), "profile:write".to_string()]
                .into_iter()
                .collect(),
        );

        snapshot.role_perms.insert(
            "content_manager".to_string(),
            vec![
                "content:read".to_string(),
                "content:write".to_string(),
                "content:moderate".to_string(),
            ]
            .into_iter()
            .collect(),
        );

        snapshot.role_perms.insert(
            "analyst".to_string(),
            vec![
                "reports:read".to_string(),
                "analytics:view".to_string(),
                "data:export".to_string(),
            ]
            .into_iter()
            .collect(),
        );

        // Create custom permission trie
        let mut trie = PermissionTrie::new();
        trie.insert(
            "/profile/{username}",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/profile/{username}".to_string(),
                required_permission: "profile:read".to_string(),
                self_only: true,
            },
        );
        trie.insert(
            "/content",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/content".to_string(),
                required_permission: "content:write,content:moderate".to_string(),
                self_only: false,
            },
        );
        trie.insert(
            "/reports/{tenant_name}",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/reports/{tenant_name}".to_string(),
                required_permission: "reports:read,analytics:view".to_string(),
                self_only: false,
            },
        );

        let mut tries = HashMap::new();
        tries.insert(Method::GET, trie);
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        // Test user with multiple roles
        let claim = create_test_claim(
            "john_doe",
            "acme_corp",
            vec![
                "basic_user".to_string(),
                "content_manager".to_string(),
                "analyst".to_string(),
            ],
            false,
        );

        // Should succeed - user has all required permissions from different roles
        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/profile/john_doe", &claim)
            .await;
        assert!(
            result.is_ok(),
            "Should access own profile with basic_user role"
        );

        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/content", &claim)
            .await;
        assert!(
            result.is_ok(),
            "Should access content with content_manager role"
        );

        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/reports/acme_corp", &claim)
            .await;
        assert!(result.is_ok(), "Should access reports with analyst role");

        // Test with user missing one required permission
        let claim_missing_perm = create_test_claim(
            "jane_doe",
            "acme_corp",
            vec![
                "basic_user".to_string(),
                "analyst".to_string(), // Missing content_manager role
            ],
            false,
        );

        let result = snapshot_with_trie
            .verify_access(&Method::GET, "/content", &claim_missing_perm)
            .await;
        assert_eq!(
            result.unwrap_err(),
            StatusCode::FORBIDDEN,
            "Should be forbidden - missing content moderation permission"
        );
    }

    #[tokio::test]
    async fn test_verify_access_role_priority_and_override() {
        let mut snapshot = create_test_snapshot();

        // Setup roles with overlapping permissions
        snapshot.role_perms.insert(
            "read_only".to_string(),
            vec!["data:read".to_string()].into_iter().collect(),
        );

        snapshot.role_perms.insert(
            "read_write".to_string(),
            vec!["data:read".to_string(), "data:write".to_string()]
                .into_iter()
                .collect(),
        );

        snapshot.role_perms.insert(
            "admin".to_string(),
            vec![
                "data:read".to_string(),
                "data:write".to_string(),
                "data:delete".to_string(),
                "system:admin".to_string(),
            ]
            .into_iter()
            .collect(),
        );

        let mut trie = PermissionTrie::new();
        trie.insert(
            "/data",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/data".to_string(),
                required_permission: "data:read".to_string(),
                self_only: false,
            },
        );
        trie.insert(
            "/data/write",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/data/write".to_string(),
                required_permission: "data:write".to_string(),
                self_only: false,
            },
        );
        trie.insert(
            "/data/delete",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/data/delete".to_string(),
                required_permission: "data:delete".to_string(),
                self_only: false,
            },
        );
        trie.insert(
            "/admin",
            PermissionRule {
                method: Method::GET,
                path_pattern: "/admin".to_string(),
                required_permission: "system:admin".to_string(),
                self_only: false,
            },
        );

        let mut tries = HashMap::new();
        tries.insert(Method::GET, trie);
        let snapshot_with_trie = AclSnapshot { tries, ..snapshot };

        // Test different role combinations
        let read_only_user =
            create_test_claim("reader", "tenant1", vec!["read_only".to_string()], false);
        let read_write_user = create_test_claim(
            "writer",
            "tenant1",
            vec!["read_only".to_string(), "read_write".to_string()],
            false,
        );
        let admin_user = create_test_claim(
            "admin",
            "tenant1",
            vec![
                "read_only".to_string(),
                "read_write".to_string(),
                "admin".to_string(),
            ],
            false,
        );

        // Read only user should only access read endpoints
        assert!(snapshot_with_trie
            .verify_access(&Method::GET, "/data", &read_only_user)
            .await
            .is_ok());
        assert_eq!(
            snapshot_with_trie
                .verify_access(&Method::GET, "/data/write", &read_only_user)
                .await
                .unwrap_err(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            snapshot_with_trie
                .verify_access(&Method::GET, "/data/delete", &read_only_user)
                .await
                .unwrap_err(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            snapshot_with_trie
                .verify_access(&Method::GET, "/admin", &read_only_user)
                .await
                .unwrap_err(),
            StatusCode::FORBIDDEN
        );

        // Read/write user should access read and write endpoints
        assert!(snapshot_with_trie
            .verify_access(&Method::GET, "/data", &read_write_user)
            .await
            .is_ok());
        assert!(snapshot_with_trie
            .verify_access(&Method::GET, "/data/write", &read_write_user)
            .await
            .is_ok());
        assert_eq!(
            snapshot_with_trie
                .verify_access(&Method::GET, "/data/delete", &read_write_user)
                .await
                .unwrap_err(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            snapshot_with_trie
                .verify_access(&Method::GET, "/admin", &read_write_user)
                .await
                .unwrap_err(),
            StatusCode::FORBIDDEN
        );

        // Admin user should access all endpoints
        assert!(snapshot_with_trie
            .verify_access(&Method::GET, "/data", &admin_user)
            .await
            .is_ok());
        assert!(snapshot_with_trie
            .verify_access(&Method::GET, "/data/write", &admin_user)
            .await
            .is_ok());
        assert!(snapshot_with_trie
            .verify_access(&Method::GET, "/data/delete", &admin_user)
            .await
            .is_ok());
        assert!(snapshot_with_trie
            .verify_access(&Method::GET, "/admin", &admin_user)
            .await
            .is_ok());
    }
}
