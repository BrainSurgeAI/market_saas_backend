use axum::{body::Body, middleware::Next, response::Response};
use hyper::http::Request;

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_id: String,
    pub client_ip: Option<String>,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            client_ip: None,
        }
    }
}

pub async fn inject_request_context(mut request: Request<Body>, next: Next) -> Response {
    let mut context = RequestContext::new();

    context.client_ip = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|header| header.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("X-Real-IP")
                .and_then(|header| header.to_str().ok())
                .map(|s| s.trim().to_string())
        });

    tracing::info!(
        request_id = %context.request_id,
        client_ip = context.client_ip.as_deref().unwrap_or("unknown"),
        "Request started: {} {}",
        request.method(),
        request.uri()
    );

    request.extensions_mut().insert(context);
    next.run(request).await
}
