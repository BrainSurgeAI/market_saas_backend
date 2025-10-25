use axum::{routing::get, Router};

use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::category_services::get_categories_tree,
};

/// Get category with sub-categories route.
pub fn categories_routes() -> Router {
    Router::new().route(
        "/api/v1/categories/tree",
        get(get_categories_tree::<MySqlRepository>),
    )
}