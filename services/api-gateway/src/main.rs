//! API Gateway
//!
//! GraphQL entry point that routes operations to backend gRPC microservices.
//! Uses `async-graphql` for the schema and `tonic` as the gRPC client.
//!
//! Optimizations:
//! - Shared gRPC channel (connection reuse / multiplexing)
//! - Circuit breaker for auth-service calls
//! - Configurable gRPC timeout
//! - GraphQL query depth and complexity limits
//! - Rate limiting, CORS, security headers
//! - Graceful shutdown on SIGTERM / SIGINT

mod circuit_breaker;
mod config;
mod grpc_client;
mod router;
mod schema;

use std::time::Duration;

use async_graphql::Schema;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

use crate::circuit_breaker::CircuitBreaker;
use crate::config::Config;
use crate::schema::{MutationRoot, QueryRoot};

/// Type alias for the gateway GraphQL schema
pub type GatewaySchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;

/// Shared application state passed to all handlers
#[derive(Clone)]
pub struct AppState {
    pub schema: GatewaySchema,
    /// Shared gRPC channel for connection reuse
    pub grpc_channel: tonic::transport::Channel,
}

/// Initialize tracing/logging based on LOG_FORMAT env var.
fn init_tracing(log_format: &str) {
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("api_gateway=debug".parse().unwrap());

    if log_format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    // Initialize tracing
    init_tracing(&config.log_format);

    info!(
        "Starting API Gateway on {}:{}",
        config.server_host, config.server_port
    );
    info!("Auth service gRPC endpoint: {}", config.auth_service_url);

    // Create a shared gRPC channel with timeout.
    // `connect_lazy()` avoids failing on startup if auth-service isn't ready yet.
    // The channel supports HTTP/2 multiplexing, so a single channel handles
    // many concurrent RPCs efficiently.
    let grpc_channel = tonic::transport::Channel::from_shared(config.auth_service_url.clone())?
        .timeout(Duration::from_secs(config.grpc_timeout_secs))
        .connect_lazy();

    info!(
        "gRPC channel configured: timeout={}s",
        config.grpc_timeout_secs
    );

    // Create circuit breaker for auth-service calls
    let cb = CircuitBreaker::new(
        config.circuit_breaker_threshold,
        Duration::from_secs(config.circuit_breaker_recovery_secs),
    );
    info!(
        "Circuit breaker configured: threshold={}, recovery={}s",
        config.circuit_breaker_threshold, config.circuit_breaker_recovery_secs
    );

    // Build GraphQL schema with shared channel, circuit breaker, and limits
    let schema = Schema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(grpc_channel.clone())
        .data(cb)
        .limit_depth(config.graphql_max_depth)
        .limit_complexity(config.graphql_max_complexity)
        .finish();

    info!(
        "GraphQL limits: max_depth={}, max_complexity={}",
        config.graphql_max_depth, config.graphql_max_complexity
    );

    // Build application state
    let state = AppState {
        schema,
        grpc_channel,
    };

    // Build router (with rate limiting + security middleware)
    let app = router::create_router(state, config.rate_limit_per_second);

    // Start server with graceful shutdown
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("API Gateway listening on {}", addr);
    info!("GraphiQL playground: http://{}/graphql", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("API Gateway shut down gracefully");
    Ok(())
}

/// Listen for SIGTERM / SIGINT (Ctrl-C) for graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, starting graceful shutdown"),
        _ = terminate => info!("Received SIGTERM, starting graceful shutdown"),
    }
}
