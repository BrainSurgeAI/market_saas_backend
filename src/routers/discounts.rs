use axum::{
    routing::{get, patch},
    Router,
};

use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::discount_services::{
        create_discount_by_tenant_and_category, get_customer_discount_list,
        update_discount_by_tenant_and_category,
    },
};

pub(super) fn discount_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/tenants/{tenant_hash}/discounts",
            get(get_customer_discount_list::<MySqlRepository>)
            .post(create_discount_by_tenant_and_category::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{tenant_hash}/discounts/{discount_id}",
            patch(update_discount_by_tenant_and_category::<MySqlRepository>),
        )
}
