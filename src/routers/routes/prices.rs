use axum::{routing::patch, Router};

use crate::{
    repositories::my_sql_repository::MySqlRepository, services::price_services::aprox_price,
};

pub fn prices_routes() -> Router {
    Router::new().route(
        "/api/v1/tenants/{tenant_hash}/prices/aprox_price",
        patch(aprox_price::<MySqlRepository>),
    )
}
