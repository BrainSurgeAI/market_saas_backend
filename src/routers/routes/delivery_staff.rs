use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::delivery_staff_serivces::{
        create_delivery_staff, disable_or_enable_delivery_staff, get_delivery_staff_by_provider,
    },
};
use axum::{
    routing::{get, patch, post},
    Router,
};
//use tower_http::compression::CompressionLayer;

pub fn delivery_staff_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/tenants/{tenant_hash}/delivery_staffs",
            post(create_delivery_staff::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/delivery_staffs",
            get(get_delivery_staff_by_provider::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/delivery_staffs/{id_card}/status",
            patch(disable_or_enable_delivery_staff::<MySqlRepository>),
        )
    // .layer(CompressionLayer::new())
}
