use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::delivery_staff_serivces::{
        create_delivery_staff, disable_or_enable_delivery_staff, get_delivery_staff_by_provider,
    },
};
use axum::{
    routing::{patch, post},
    Router,
};
//use tower_http::compression::CompressionLayer;

pub(super) fn delivery_staff_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/deliveries",
            post(create_delivery_staff::<MySqlRepository>)
                .get(get_delivery_staff_by_provider::<MySqlRepository>),
        )
        .route(
            "/api/v1/deliveries/{id_card}/status",
            patch(disable_or_enable_delivery_staff::<MySqlRepository>),
        )
    // .layer(CompressionLayer::new())
}
