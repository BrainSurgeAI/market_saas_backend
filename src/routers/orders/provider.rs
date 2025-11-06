use axum::{
    routing::{get, patch, put},
    Router,
};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::order_services::get_provider_today_product_order_summary;
use crate::services::orders::provider::{
    deliver_to_market, start_exchange_preparing, start_preparing, update_exchange_item_actual_quantity,
};

pub(super) fn provider_routes() -> Router {
    Router::new()
        .route(
            "/{order_code}/preparing",
            patch(start_preparing::<MySqlRepository>),
        )
        .route(
            "/{order_code}/start-exchange-preparing",
            patch(start_exchange_preparing::<MySqlRepository>),
        )
        .route(
            "/{order_code}/update-exchange-item-actual",
            patch(update_exchange_item_actual_quantity::<MySqlRepository>),
        )
        .route(
            "/{order_code}/deliver-to-market",
            put(deliver_to_market::<MySqlRepository>),
        )
        .route(
            "/api/v1/providers/{provider_hash}/orders/today-summary",
            get(get_provider_today_product_order_summary::<MySqlRepository>),
        )
}
