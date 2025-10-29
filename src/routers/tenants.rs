use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::tenant_services::{
        create_tenant, disable_tenant, get_all_providers_by_market, get_tenant_by_user,
        get_tenant_detail_by_self, get_tenant_financials, get_tenant_users,
        list_tenants_by_market, update_tenant_by_market, update_tenant_by_self,
        get_tenant_detail_by_hashed_name, get_users
    },
};

use axum::{
    routing::{get, patch, post},
    Router,
};

pub(super) fn tenant_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/tenants/me",
            get(get_tenant_detail_by_self::<MySqlRepository>),
        )
        .route(
            "/api/v1/users/{username}/tenants",
            get(get_tenant_by_user::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants",
            post(create_tenant::<MySqlRepository>)
            .get(list_tenants_by_market::<MySqlRepository>),
        )
        .route(
            "/api/v1/financials",
            get(get_tenant_financials::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{hashed_name}/users",
            get(get_tenant_users::<MySqlRepository>),
        )
        .route("/api/v1/users", get(get_users::<MySqlRepository>))
        .route(
            "/api/v1/tenants/me",
            patch(update_tenant_by_self::<MySqlRepository>),
        )
        .route(
            "/api/v1/providers",
            get(get_all_providers_by_market::<MySqlRepository>),
        )
        .route(
            "/api/v1/markets/{market_hash}/tenants/{tenant_hash}/status",
            patch(disable_tenant::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}",
            patch(update_tenant_by_market::<MySqlRepository>)
            .get(get_tenant_detail_by_hashed_name::<MySqlRepository>),
        )
}
