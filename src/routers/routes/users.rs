use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::{
        auth_service::reset_password,
        notification_services::{list_notifications, update_notification_status},
        user_services::{create_tenant_user, delete_user, enable_user, get_user, update_user},
    },
};
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

pub fn user_routes() -> Router {
    Router::new()
        .route(
            "/api/v1/users/{username}/notifications",
            get(list_notifications),
        )
        .route(
            "/api/v1/users/{username}/notifications/{message_id}/status",
            patch(update_notification_status),
        )
        .route("/api/v1/users/{username}", get(get_user::<MySqlRepository>))
        .route(
            "/api/v1/users/{username}",
            patch(update_user::<MySqlRepository>),
        )
        .route(
            "/api/v1/users/{username}/password",
            patch(reset_password::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{hashed_name}/users/{username}/disable",
            delete(delete_user::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{hashed_name}/users/{username}/enable",
            patch(enable_user::<MySqlRepository>),
        )
        .route(
            "/api/v1/tenants/{hashed_name}/users",
            post(create_tenant_user::<MySqlRepository>),
        )
}
