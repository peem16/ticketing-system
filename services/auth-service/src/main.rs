//! Auth Service - JWT-based authentication microservice
//!
//! This service handles user registration, login, and JWT token management.
//! Built with Clean Architecture principles.
//! Exposes both HTTP (Axum) and gRPC (Tonic) interfaces.

mod application;
mod domain;
mod infrastructure;
mod interface;

use std::sync::Arc;

use axum::Router;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tokio::net::TcpListener;
use tokio::signal;
use tonic::transport::Server as TonicServer;
use tracing::info;

use crate::infrastructure::{
    cache::token_cache::CachedTokenService,
    config::Config,
    db::connection::create_connection_pool,
    security::{argon2_password_hasher::Argon2PasswordHasher, jwt_token_service::JwtTokenService},
};
use crate::interface::grpc::service::pb::auth_service_server::AuthServiceServer;
use crate::interface::grpc::service::AuthServiceGrpc;
use crate::interface::http;

/// Embedded database migrations — compiled into the binary so no external
/// diesel CLI is needed at runtime.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Application state shared across handlers
pub struct AppState {
    pub pool: infrastructure::db::connection::DbPool,
    pub password_hasher: Arc<dyn domain::auth::PasswordHasher + Send + Sync>,
    pub token_service: Arc<dyn domain::auth::TokenService + Send + Sync>,
}

/// Initialize tracing/logging based on LOG_FORMAT env var.
///
/// - `"json"` → structured JSON output (production)
/// - anything else → human-readable output (development)
fn init_tracing(log_format: &str) {
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("auth_service=debug".parse().unwrap());

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

/// Run embedded Diesel migrations on startup.
fn run_migrations(pool: &infrastructure::db::connection::DbPool) {
    let mut conn = pool
        .get()
        .expect("Failed to get database connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
    info!("Database migrations completed successfully");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    // Initialize tracing
    init_tracing(&config.log_format);

    info!(
        "Starting Auth Service - HTTP on {}:{}, gRPC on {}:{}",
        config.server_host, config.server_port, config.server_host, config.grpc_port
    );

    // Initialize database connection pool (configurable size)
    let pool = create_connection_pool(&config.database_url, config.db_pool_max_size)?;

    // Run embedded migrations
    run_migrations(&pool);

    // Initialize services
    let password_hasher = Arc::new(Argon2PasswordHasher::new());
    let jwt_service = Arc::new(JwtTokenService::new(
        config.jwt_secret.clone(),
        config.jwt_expiration_secs,
    ));

    // Wrap token service with moka cache (if configured)
    let token_service: Arc<dyn domain::auth::TokenService + Send + Sync> =
        if config.token_cache_ttl_secs > 0 {
            info!(
                "Token validation cache enabled: TTL={}s, max_capacity={}",
                config.token_cache_ttl_secs, config.token_cache_max_capacity
            );
            Arc::new(CachedTokenService::new(
                jwt_service,
                config.token_cache_ttl_secs,
                config.token_cache_max_capacity,
            ))
        } else {
            jwt_service
        };

    // Build application state
    let state = Arc::new(AppState {
        pool,
        password_hasher,
        token_service,
    });

    // Build HTTP router (with rate limiting + security middleware)
    let app: Router = http::router::create_router(Arc::clone(&state), config.rate_limit_per_second);

    // Build gRPC service
    let grpc_service = AuthServiceGrpc::new(Arc::clone(&state));

    // Start HTTP server
    let http_addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&http_addr).await?;
    info!("Auth Service HTTP listening on {}", http_addr);

    let http_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    });

    // Start gRPC server
    let grpc_addr = format!("{}:{}", config.server_host, config.grpc_port)
        .parse()
        .expect("Invalid gRPC address");
    info!("Auth Service gRPC listening on {}", grpc_addr);

    let grpc_handle = tokio::spawn(async move {
        TonicServer::builder()
            .add_service(AuthServiceServer::new(grpc_service))
            .serve_with_shutdown(grpc_addr, shutdown_signal())
            .await
            .unwrap();
    });

    // Wait for both servers
    tokio::select! {
        _ = http_handle => info!("HTTP server stopped"),
        _ = grpc_handle => info!("gRPC server stopped"),
    }

    info!("Auth Service shut down gracefully");
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
