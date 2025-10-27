use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::price_services::{
        approve_price, batch_create_product_price, get_pricer_daily_price_comparison,
        get_user_pending_price_products,
    },
};
use axum::{
    routing::{get, patch},
    Router,
};

pub(super) fn prices_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/product_prices",
            get(get_pricer_daily_price_comparison::<MySqlRepository>)
                .post(batch_create_product_price::<MySqlRepository>),
        )
        .route(
            "/api/v1/product_prices/status",
            get(get_user_pending_price_products::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{hashed_name}/prices/approve-price",
            patch(approve_price::<MySqlRepository>),
        )
}
