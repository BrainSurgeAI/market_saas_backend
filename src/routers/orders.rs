use axum::{
    routing::{get, patch, post, put},
    Router,
};

use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::order_services::{
        create_order, dispatch_order, get_after_sale_orders_by_provider, get_order_by_order_code,
        get_orders, get_provider_today_product_order_summary, return_exchange_order,
        update_actual_quantity, update_order_status, update_order_status_to_processing,
    },
};

pub(super) fn order_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/customers/{customer_hash}/orders",
            post(create_order::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/orders",
            get(get_orders::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/orders/{order_code}",
            get(get_order_by_order_code::<MySqlRepository>),
        )
        .route(
            "/api/v1/markets/{market_hash}/orders/{order_code}/dispatch",
            patch(dispatch_order::<MySqlRepository>),
        )
        .route(
            "/api/v1/providers/{provider_hash}/orders/{order_code}/processing",
            patch(update_order_status_to_processing::<MySqlRepository>),
        )
        .route(
            "/api/v1/providers/{provider_hash}/orders/{order_code}/update-quantities",
            put(update_actual_quantity::<MySqlRepository>),
        )
        .route(
            "/api/v1/customers/{customer_hash}/orders/{order_code}/operations",
            post(return_exchange_order::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/orders/{order_code}/operations",
            patch(update_order_status::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/orders/after_sale",
            get(get_after_sale_orders_by_provider::<MySqlRepository>),
        )
        .route(
            "/api/v1/providers/{provider_hash}/orders/today-summary",
            get(get_provider_today_product_order_summary::<MySqlRepository>),
        )
}
