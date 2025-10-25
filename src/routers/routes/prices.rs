use axum::{routing::{patch, get}, Router};

use crate::{
    repositories::my_sql_repository::MySqlRepository, 
    services::price_services::{aprox_price, get_pricer_published_prices},
};

pub fn prices_routes() -> Router {
    Router::new()
    .route(
        "/api/v1/tenants/{tenant_hash}/prices/aprox_price",
        patch(aprox_price::<MySqlRepository>),
    )
    .route(
        "/api/v1/product_prices",
        get(get_pricer_published_prices::<MySqlRepository>),
    )
}