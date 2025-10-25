use std::{sync::Arc, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use hyper::{header, Method};
use tower_http::cors::CorsLayer;
use sqlx::MySqlPool;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use crate::{
    common::AppError,
    dto,
    middleware::{
        auth::{AppState, auth_middleware}, context::inject_request_context, logging_layer::LoggingLayer, 
    },
    repositories::{
        my_sql_repository::MySqlRepository, 
        system_log_repo::MySqlSystemLogRepository,
        
    },
    services::{
        auth_service::{login, register},
        price_services::get_price_announcements,
        category_services::get_level_one_categories,
        superadmin_services::super_admin_login,
        system_log_service::SystemLogService,
    },
};

use crate::routers::routes::{
    order_routes, roles_routes, tenant_routes, user_routes, workspace_routes,
};

use super::{
    categories_routes, delivery_staff_routes, discount_routes, prices_routes, products_routes,
    reconciliation_statement_routes,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::services::auth_service::login,
        crate::services::auth_service::register,
        crate::services::user_services::update_user,
        crate::services::user_services::delete_user,
        crate::services::user_services::get_user,
    ),
    components(
        schemas(
            dto::auth::LoginRequest,
            dto::auth::RegisterRequest,
            dto::auth::ResetPasswordRequest,
            dto::users::UserCreateDto,
        )
    ),
    tags(
        (name = "auth", description = "认证相关接口"),
        (name = "users", description = "用户管理接口"),
        (name = "tenants", description = "租户管理接口")
    ),
    info(
        title = "My API",
        version = "1.0",
        description = "API Documentation"
    )
)]
pub struct ApiDoc;

fn configure_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(header::HeaderValue::from_static("*"))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::HEAD,
            Method::PATCH,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
}

fn configure_routes(pool: &MySqlPool) -> Router {
    let api_doc = ApiDoc::openapi();

    let public_routes = Router::new()
        .route("/api/v1/register", post(register::<MySqlRepository>))
        .route("/api/v1/login", post(login::<MySqlRepository>))
        .route(
            "/api/v1/superadmin/login",
            post(super_admin_login::<MySqlRepository, MySqlRepository>),
        )
        .route(
            "/api/v1/categories",
            get(get_level_one_categories::<MySqlRepository>),
        )
        .route(
            "/api/v1/price_announcements",
            get(get_price_announcements::<MySqlRepository>),
        )
        .route(
            "/api/v1/openapi.json",
            get(|| async move { Json(api_doc.clone()) }),
        )
        .merge(Redoc::with_url("/api/v1/redoc", "/api/v1/openapi.json"));

    let log_repo = Arc::new(MySqlSystemLogRepository::new(Arc::new(pool.clone())));
    let log_service = Arc::new(SystemLogService::new(log_repo));
    let logging_layer = LoggingLayer::new(Arc::clone(&log_service));


    let mysql = Arc::new(MySqlRepository::new(pool.clone()));

    let shared_state = AppState {
        repo: mysql.clone(),
    };
    // Protected routes (requires authentication)
    let protected_routes = Router::new()
        .merge(workspace_routes())
        .merge(user_routes())
        .merge(tenant_routes())
        .merge(roles_routes())
        .merge(products_routes())
        .merge(prices_routes())
        .merge(order_routes())
        .merge(categories_routes())
        .merge(delivery_staff_routes())
        .merge(reconciliation_statement_routes())
        .merge(discount_routes())
        .layer(logging_layer)
        .layer(axum::middleware::from_fn_with_state(shared_state.clone(), auth_middleware));

        public_routes.with_state(shared_state).merge(protected_routes)
}

pub(crate) fn create_router(pool: &MySqlPool) -> Router {
    let cors = configure_cors();
    let routes = configure_routes(pool);
    
    let my_sql_repository = MySqlRepository::new(pool.clone());
    Router::new()
        .merge(routes)
        .layer(cors)
        .layer(axum::middleware::from_fn(inject_request_context))
        .layer(Extension(pool.clone()))
        .layer(Extension(my_sql_repository.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .timeout(Duration::from_secs(30))
                .into_inner(),
        )
}

async fn handle_error(error: Box<dyn std::error::Error + Send + Sync>) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return AppError::Internal("Request timeout".into()).into_response();
    }

    AppError::Internal("Internal server error".into()).into_response()
}