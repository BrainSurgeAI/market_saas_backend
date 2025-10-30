use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::order_services::{
        create_order, assign_order_to_provider, get_after_sale_orders_by_provider, get_order_by_order_code,
        get_orders, get_provider_today_product_order_summary, inspect_sub_orders,
        begin_inspect_order, start_preparing, deliver_to_market, accept_order, deliver_to_customer,
        cancel_order,
    },
};

pub(super) fn order_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/orders",
            post(create_order::<MySqlRepository>)
            .get(get_orders::<MySqlRepository>),
        )
        .route(
            "/api/v1/orders/{order_code}",
            get(get_order_by_order_code::<MySqlRepository>),
        )
        .route(
            "/api/v1/orders/{order_code}/assign",
            patch(assign_order_to_provider::<MySqlRepository>),
        )
        .route(
            "/api/v1/orders/{order_code}/preparing",  // 供应商备货
            patch(start_preparing::<MySqlRepository>),
        )
        .route(
            "/api/v1/orders/{order_code}/deliver-to-market",
            put(deliver_to_market::<MySqlRepository>),
        )
        .route(
            "/api/v1/orders/{order_code}/inspect-sub-orders",
            patch(inspect_sub_orders::<MySqlRepository>),
        )
        .route(
            "/api/v1/orders/{order_code}/begin-inspect-order",
            patch(begin_inspect_order::<MySqlRepository>),
        ).route(
            "/api/v1/orders/{order_code}/accept",
            patch(accept_order::<MySqlRepository>),
        )
        .route(
            "/api/v1/orders/{order_code}/deliver-to-customer",
            patch(deliver_to_customer::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/orders/after_sale",
            get(get_after_sale_orders_by_provider::<MySqlRepository>),
        )
        .route(
            "/api/v1/orders/{order_code}/cancel",
            delete(cancel_order::<MySqlRepository>),
        )
        .route(
            "/api/v1/providers/{provider_hash}/orders/today-summary",
            get(get_provider_today_product_order_summary::<MySqlRepository>),
        )
}
