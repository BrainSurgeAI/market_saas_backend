mod middleware;
mod models;
mod repositories;
mod routers;
mod services;
mod types;
mod utils;
mod acl_core;
mod common;
mod dto;


use dashmap::DashMap;
use dotenv::dotenv;
use hyper::Method;
use crate::acl_core::{
    acl_snapshot::AclSnapshot,
    permission_trie::{PermissionRule, PermissionTrie},
};
use crate::{
    acl_core::acl_snapshot::ACL_SNAPSHOT,
    routers::router_config::create_router
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

use tracing::{info, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}={},tower_http=info", env!("CARGO_CRATE_NAME"), log_level).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    info!(
        "Database connected with max connections {}",
        pool.options().get_max_connections()
    );

    init_acl_snapshot(&pool)
        .await
        .expect("Failed to initialize ACL snapshot");

    let snapshot = ACL_SNAPSHOT.load();
    debug!(
        "ACL Snapshot initialized {:?}",
        snapshot
    );

    let app = create_router(&pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Server running at http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct RuleConfig {
    pub role: String,
    pub path_pattern: String,
    pub http_method: String,
    pub required_permission: String,
    pub self_only: bool,
}

pub async fn init_acl_snapshot(pool: &MySqlPool) -> Result<(), anyhow::Error> {
    let rules = sqlx::query_as::<_, RuleConfig>(
        r#"
        SELECT 
            r.name              AS role,
            p.path_pattern,
            p.http_method,
            p.name              AS required_permission,
            COALESCE(p.self_only, 0) AS self_only
        FROM role_permissions rp
        JOIN roles r              ON rp.role_id = r.id
        JOIN permissions p        ON rp.permission_id = p.id
        where p.path_pattern IS NOT NULL AND p.http_method IS NOT NULL -- only for test
        ORDER BY role, p.http_method, p.path_pattern
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut grouped: HashMap<String, Vec<RuleConfig>> = HashMap::new();

    for r in rules {
        grouped.entry("market_saas".to_string()).or_default().push(r);
    }

    for (_role, rs) in grouped {
        let mut tries_by_method: HashMap<Method, PermissionTrie> = HashMap::new();
        let mut role_perms: HashMap<String, HashSet<String>> = HashMap::new();

        for rule in rs {
            // tries
            let method = rule.http_method.parse::<Method>()?;
            let trie = tries_by_method
                .entry(method.clone())
                .or_insert_with(PermissionTrie::new);
            trie.insert(
                &rule.path_pattern,
                PermissionRule {
                    method,
                    required_permission: rule.required_permission.clone(),
                    self_only: rule.self_only,
                    path_pattern: rule.path_pattern.clone(),
                },
            );

            // role -> permission 归集
            role_perms
                .entry(rule.role)
                .or_insert_with(HashSet::new)
                .insert(rule.required_permission);
        }

        let snapshot = AclSnapshot {
            tries: tries_by_method,
            role_perms,
            version: 1,
            role_cache: Arc::new(DashMap::new()),
        };

        ACL_SNAPSHOT.store(Arc::new(snapshot));
    }

    Ok(())
}
