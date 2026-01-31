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
use tokio::net::TcpListener;
use tonic::transport::Server as TonicServer;
use tracing::info;

use crate::infrastructure::{
    config::Config,
    db::connection::create_connection_pool,
    security::{argon2_password_hasher::Argon2PasswordHasher, jwt_token_service::JwtTokenService},
};
use crate::interface::grpc::service::pb::auth_service_server::AuthServiceServer;
use crate::interface::grpc::service::AuthServiceGrpc;
use crate::interface::http;

/// Application state shared across handlers
pub struct AppState {
    pub pool: infrastructure::db::connection::DbPool,
    pub password_hasher: Arc<dyn domain::auth::PasswordHasher + Send + Sync>,
    pub token_service: Arc<dyn domain::auth::TokenService + Send + Sync>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("auth_service=debug".parse().unwrap()),
        )
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    info!(
        "Starting Auth Service - HTTP on {}:{}, gRPC on {}:{}",
        config.server_host, config.server_port, config.server_host, config.grpc_port
    );

    // Initialize database connection pool
    let pool = create_connection_pool(&config.database_url)?;

    // Initialize services
    let password_hasher = Arc::new(Argon2PasswordHasher::new());
    let token_service = Arc::new(JwtTokenService::new(
        config.jwt_secret.clone(),
        config.jwt_expiration_secs,
    ));

    // Build application state
    let state = Arc::new(AppState {
        pool,
        password_hasher,
        token_service,
    });

    // Build HTTP router
    let app: Router = http::router::create_router(Arc::clone(&state));

    // Build gRPC service
    let grpc_service = AuthServiceGrpc::new(Arc::clone(&state));

    // Start HTTP server
    let http_addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&http_addr).await?;
    info!("Auth Service HTTP listening on {}", http_addr);

    let http_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Start gRPC server
    let grpc_addr = format!("{}:{}", config.server_host, config.grpc_port)
        .parse()
        .expect("Invalid gRPC address");
    info!("Auth Service gRPC listening on {}", grpc_addr);

    let grpc_handle = tokio::spawn(async move {
        TonicServer::builder()
            .add_service(AuthServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await
            .unwrap();
    });

    // Wait for both servers
    tokio::select! {
        _ = http_handle => info!("HTTP server stopped"),
        _ = grpc_handle => info!("gRPC server stopped"),
    }

    Ok(())
}
