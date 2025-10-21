use hyper::header;
use hyper::Method;
use tower_http::cors::CorsLayer;

pub fn configure_cors() -> CorsLayer {
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
