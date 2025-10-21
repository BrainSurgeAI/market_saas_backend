use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::reconciliation_services::{
        create_reconciliation_statement, get_statement_orders_by_id,
        get_statements_by_tenant_and_date,
    },
};

pub fn reconciliation_statement_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/reconciliation_statements",
            post(create_reconciliation_statement::<MySqlRepository>),
        )
        .route(
            "/api/v1/reconciliation_statements",
            get(get_statements_by_tenant_and_date::<MySqlRepository>),
        )
        .route(
            "/api/v1/reconciliation_statements/{id}",
            get(get_statement_orders_by_id::<MySqlRepository>),
        )
}
