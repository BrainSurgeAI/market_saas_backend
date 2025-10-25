use crate::{
    repositories::my_sql_repository::MySqlRepository,
    services::{
        auth_service::reset_password,
        notification_services::{list_notifications, read_notification},
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
            "/api/v1/notifications",
            get(list_notifications)
        )
        .route(
            "/api/v1/notifications/{message_id}",
            patch(read_notification)
        )
        .route("/api/v1/users/me", get(get_user::<MySqlRepository>))
        .route(
            "/api/v1/users/{username}",
            patch(update_user::<MySqlRepository>),
        )
        .route(
            "/api/v1/password/reset",
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
