use axum::{routing::patch, Router};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::orders::marketplace::{assign_order_to_provider, complete_inspection, deliver_to_customer};

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
        .route("/{order_code}/complete-inspection", patch(complete_inspection::<MySqlRepository>))
}
