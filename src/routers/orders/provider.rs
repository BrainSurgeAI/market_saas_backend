use axum::{
    routing::{get, patch, post, put},
    Router,
};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::order_services::{
    get_provider_dashboard_stats, get_provider_preparation_summary,
    get_provider_today_delivered_products,
};
use crate::services::orders::provider::{
    deliver_to_market, deliver_exchange_to_market, accept_after_sales_request, 
    start_preparing, update_exchange_item_quantity, get_order_delivery_history,
};

pub(super) fn provider_routes() -> Router {
    Router::new()
        .route(
            "/{order_code}/start-preparing",
            post(start_preparing::<MySqlRepository>),
        )
        .route(
            "/{order_code}/accept-after-sales",
            post(accept_after_sales_request::<MySqlRepository>),
        )
        .route(
            "/{order_code}/exchange-items/{order_detail_id}",
            patch(update_exchange_item_quantity::<MySqlRepository>),
        )
        .route(
            "/{order_code}/deliver-to-market",
            put(deliver_to_market::<MySqlRepository>),
        )
        .route(
            "/preparation-summary",
            get(get_provider_preparation_summary::<MySqlRepository>),
        )
        .route(
            "/dashboard-stats",
            get(get_provider_dashboard_stats::<MySqlRepository>),
        )
        .route(
            "/today-delivered-products",
            get(get_provider_today_delivered_products::<MySqlRepository>),
        )
        .route(
            "/{order_code}/deliver-exchange-to-market",
            patch(deliver_exchange_to_market::<MySqlRepository>),
        )
        .route(
            "/{order_code}/delivery-history",
            get(get_order_delivery_history::<MySqlRepository>),
        )
}
