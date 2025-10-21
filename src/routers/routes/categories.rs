use axum::{routing::get, Router};

use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::category_services::get_category_with_sub_categories,
};

pub fn categories_routes() -> Router {
    Router::new().route(
        "/api/v1/category_with_sub_categories",
        get(get_category_with_sub_categories::<MySqlRepository>),
    )
}
