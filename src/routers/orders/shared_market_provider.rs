use axum::{
    routing::{delete, get, patch},
    Router,
};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::order_services::get_after_sale_orders_by_provider;
use crate::services::orders::shared_market_customer::{
    accept_order, begin_exchange_inspect_order, begin_inspect_order, cancel_order,
    exchange_request, inspect_sub_orders, return_order,
};

pub(super) fn shared_market_provider_routes() -> Router {
    Router::new()
        .route(
            "/{order_code}/inspect-sub-orders",
            patch(inspect_sub_orders::<MySqlRepository>),
        )
        .route(
            "/{order_code}/begin-inspect-order",
            patch(begin_inspect_order::<MySqlRepository>),
        )
        .route(
            "/{order_code}/accept",
            patch(accept_order::<MySqlRepository>),
        )
        .route(
            "/{order_code}/exchange-request",
            patch(exchange_request::<MySqlRepository>),
        )
        .route(
            "/{order_code}/begin-exchange-inspect-order",
            patch(begin_exchange_inspect_order::<MySqlRepository>),
        )
        .route(
            "/{order_code}/return",
            patch(return_order::<MySqlRepository>),
        )
        .route(
            "/{order_code}/cancel",
            delete(cancel_order::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/orders/after_sale",
            get(get_after_sale_orders_by_provider::<MySqlRepository>),
        )
}
