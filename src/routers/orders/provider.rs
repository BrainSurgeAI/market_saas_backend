use axum::{
    routing::{get, patch, post, put},
    Router,
};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::order_services::get_provider_preparation_summary;
use crate::services::orders::provider::{
    deliver_to_market, deliver_exchange_to_market, accept_after_sales_request, start_preparing, update_exchange_item_quantity,
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
            "/api/v1/orders/preparation-summary",
            get(get_provider_preparation_summary::<MySqlRepository>),
        )
        .route(
            "/{order_code}/deliver-exchange-to-market",
            patch(deliver_exchange_to_market::<MySqlRepository>),
        )
}
