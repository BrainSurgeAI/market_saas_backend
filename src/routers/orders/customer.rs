use axum::{
    routing::post,
    Router,
};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::orders::customer::create_order;

pub(super) fn customer_routes() -> Router {
    Router::new()
        .route("/", post(create_order::<MySqlRepository>))
        // .route(
        //     "/{order_code}/deliver-exchange-to-market",
        //     patch(exchange_deliver_to_market::<MySqlRepository>),
        // )
      
}
