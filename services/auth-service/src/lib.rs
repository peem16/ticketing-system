//! Auth Service Library
//!
//! This module exposes the service's internal modules for testing purposes.

use std::sync::Arc;

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;

/// Application state shared across handlers
pub struct AppState {
    pub pool: infrastructure::db::connection::DbPool,
    pub password_hasher: Arc<dyn domain::auth::PasswordHasher + Send + Sync>,
    pub token_service: Arc<dyn domain::auth::TokenService + Send + Sync>,
}
