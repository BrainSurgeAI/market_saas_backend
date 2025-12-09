use axum::{
    routing::{get, post},
    Router,
};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::orders::customer::{create_order, get_customer_statistics};

pub(super) fn customer_routes() -> Router {
    Router::new()
        .route("/", post(create_order::<MySqlRepository>))
        .route("/statistics", get(get_customer_statistics::<MySqlRepository>))
}
