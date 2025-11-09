use axum::{routing::patch, Router};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::orders::marketplace::{accept_order, assign_order_to_provider, deliver_to_customer};

pub(super) fn marketplace_routes() -> Router {
    Router::new()
        .route(
            "/{order_code}/assign",
            patch(assign_order_to_provider::<MySqlRepository>),
        )
        .route(
            "/{order_code}/deliver-to-customer",
            patch(deliver_to_customer::<MySqlRepository>),
        )
        .route("/{order_code}/accept", patch(accept_order::<MySqlRepository>))
}
