use axum::{routing::get, Router};
use hyper::StatusCode;

async fn workspace_route() -> Result<String, StatusCode> {
    Ok("Welcome".to_string())
}

pub fn workspace_routes() -> Router {
    Router::new().route("/api/v1/workspace", get(workspace_route))
}
