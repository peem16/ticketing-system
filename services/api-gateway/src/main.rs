//! API Gateway
//!
//! HTTP entry point that routes requests to backend gRPC microservices.
//! Currently proxies auth endpoints to the auth-service.

mod config;
mod grpc_client;
mod handlers;
mod router;

use tokio::net::TcpListener;
use tracing::info;

use crate::config::Config;

/// Shared application state passed to all handlers
#[derive(Clone)]
pub struct AppState {
    pub auth_service_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("api_gateway=debug".parse().unwrap()),
        )
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    info!(
        "Starting API Gateway on {}:{}",
        config.server_host, config.server_port
    );
    info!("Auth service gRPC endpoint: {}", config.auth_service_url);

    // Build application state
    let state = AppState {
        auth_service_url: config.auth_service_url,
    };

    // Build router
    let app = router::create_router(state);

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("API Gateway listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
