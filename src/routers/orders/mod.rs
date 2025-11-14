mod common;
mod customer;
mod marketplace;
mod provider;
mod shared_market_customer;

use axum::Router;

pub(super) fn order_routes() -> Router {
    Router::new().nest(
        "/api/v1/orders",
        Router::new()
            .merge(common::common_routes())
            .merge(customer::customer_routes())
            .merge(marketplace::marketplace_routes())
            .merge(provider::provider_routes())
            .merge(shared_market_customer::shared_market_customer_routes()),
    )
}
