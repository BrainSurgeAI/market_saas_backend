use axum::{routing::{patch, get}, Router};

use crate::{
    repositories::my_sql_repository::MySqlRepository, 
    services::price_services::{
        aprox_price, 
        get_pricer_published_prices,
        batch_create_product_price,
    },
};

pub(crate) fn prices_routes() -> Router {
    Router::new()
    .route(
        "/api/v1/prices/aprox_price",
        patch(aprox_price::<MySqlRepository>),
    )
    .route(
        "/api/v1/product_prices",
        get(get_pricer_published_prices::<MySqlRepository>)
        .post(batch_create_product_price::<MySqlRepository>),
    )
}