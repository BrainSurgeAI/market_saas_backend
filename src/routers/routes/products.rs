use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::product_services::{
        get_processing_fees,
        get_product_detail, 
        get_product_list, 
        get_product_price_status_stats,
        get_products_overview, 
        update_product, 
        update_product_status,
    },
};
use axum::{
    routing::{delete, get, patch, put},
    Router,
};

pub(crate) fn products_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/product_price_status_stats",
            get(get_product_price_status_stats::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/products/{product_code}",
            get(get_product_detail::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/products/{product_code}",
            put(update_product::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/products/{product_code}/archive",
            delete(update_product_status::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/products/{product_code}/enable",
            patch(update_product_status::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/products",
            get(get_products_overview::<MySqlRepository>),
        )
        .route(
            "/api/v1/customers/{tenant_hash}/products",
            get(get_product_list::<MySqlRepository>),
        )
        .route(
            "/api/v1/processing-fees",
            get(get_processing_fees::<MySqlRepository>),
        )
}
