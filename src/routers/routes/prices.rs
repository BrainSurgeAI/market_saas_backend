use axum::{routing::get, Router};
use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::price_services::{
        get_pricer_daily_price_comparison,
        get_user_pending_price_products,
        batch_create_product_price,
    },
};

pub(crate) fn prices_routes() -> Router {
    Router::new()
    .route(
        "/api/v1/product_prices",
        get(get_pricer_daily_price_comparison::<MySqlRepository>)
        .post(batch_create_product_price::<MySqlRepository>),
    )
    .route(
        "/api/v1/product_prices/status",
        get(get_user_pending_price_products::<MySqlRepository>)
    )
  }