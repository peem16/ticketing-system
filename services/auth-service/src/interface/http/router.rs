//! HTTP router configuration

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use super::handlers;
use crate::AppState;

/// Create the application router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Auth routes
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/me", get(handlers::me))
        // Health check
        .route("/health", get(handlers::health))
        // Middleware
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state)
}
