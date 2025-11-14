use axum::{
    routing::post,
    Router,
};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::orders::customer::create_order;

pub(super) fn customer_routes() -> Router {
    Router::new()
        .route("/", post(create_order::<MySqlRepository>))      
}
