use crate::{
    middleware::admin_check::require_super_admin,
    repositories::my_sql_repository::MySqlRepository,
    services::{
        role_services::{
            get_permissions, get_permissions_by_role_id, get_role_by_id, get_roles,
            get_roles_by_tenant_type, update_role_by_id, update_role_permissions,
        },
        superadmin_services::reset_password,
    },
};
use axum::{
    middleware::from_fn,
    routing::{get, patch, post, put},
    Router,
};

pub fn roles_routes() -> Router {
    let admin_routes = Router::new()
        .route("/api/v1/roles", get(get_roles::<MySqlRepository>))
        .route(
            "/api/v1/permissions",
            get(get_permissions::<MySqlRepository>),
        )
        .route(
            "/api/v1/roles/{role_id}/permissions",
            get(get_permissions_by_role_id::<MySqlRepository>),
        )
        .route(
            "/api/v1/roles/{role_id}/permissions",
            post(update_role_permissions::<MySqlRepository>),
        )
        .route(
            "/api/v1/roles/{role_id}",
            put(update_role_by_id::<MySqlRepository>),
        )
        .route(
            "/api/v1/roles/{role_id}",
            get(get_role_by_id::<MySqlRepository>),
        )
        .route(
            "/api/v1/superadmin/password",
            patch(reset_password::<MySqlRepository>),
        )
        .layer(from_fn(require_super_admin));

    Router::new()
        .route(
            "/api/v1/tenants/{tenant_name}/types/{tenant_type}/roles",
            get(get_roles_by_tenant_type::<MySqlRepository>),
        )
        .merge(admin_routes)
}
