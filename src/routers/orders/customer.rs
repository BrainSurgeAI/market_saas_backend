use axum::{
    routing::{get, post},
    Router,
};

use crate::repositories::my_sql_repository::MySqlRepository;
use crate::services::orders::customer::{create_order, get_dashboard_stats};

pub(super) fn customer_routes() -> Router {
    Router::new()
        .route("/", post(create_order::<MySqlRepository>))
        .route("/dashboard", get(get_dashboard_stats::<MySqlRepository>))
}
