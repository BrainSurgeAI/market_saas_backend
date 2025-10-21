use dotenv::dotenv;
use sqlx::MySqlPool;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use market_saas_backend::{libs::permission::init_permissions, routers::router::create_router};

#[tokio::main]
async fn main() {
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}={},tower_http=info", env!("CARGO_CRATE_NAME"), log_level).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    info!(
        "Database connected with max connections {}",
        pool.options().get_max_connections()
    );

    let config_path =
        std::env::var("PERMISSIONS_CONFIG_PATH").unwrap_or_else(|_| "permissions.yaml".to_string());
    init_permissions(&config_path)
        .await
        .expect("Failed to load initial permissions");

    let app = create_router(&pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Server running at http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
