//! HTTP router configuration with security middleware

use std::num::NonZeroU32;
use std::sync::Arc;

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Extension, Router,
};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

use super::handlers;
use crate::AppState;

/// Shared rate limiter type
type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Rate-limiting middleware
///
/// Returns 429 Too Many Requests when the global rate limit is exceeded.
async fn rate_limit_middleware(
    Extension(limiter): Extension<SharedRateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match limiter.check() {
        Ok(_) => Ok(next.run(request).await),
        Err(_) => Err(StatusCode::TOO_MANY_REQUESTS),
    }
}

/// Create the application router with all routes and security middleware
pub fn create_router(state: Arc<AppState>, rate_limit_per_second: u32) -> Router {
    let mut router = Router::new()
        // Auth routes
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/auth/me", get(handlers::me))
        // Health check
        .route("/health", get(handlers::health));

    // Rate limiting (if configured)
    if rate_limit_per_second > 0 {
        if let Some(rps) = NonZeroU32::new(rate_limit_per_second) {
            let quota = Quota::per_second(rps);
            let limiter: SharedRateLimiter = Arc::new(RateLimiter::direct(quota));
            router = router
                .layer(Extension(limiter))
                .layer(middleware::from_fn(rate_limit_middleware));
        }
    }

    router
        // Request body size limit (1 MB)
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        // Security headers
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-store"),
        ))
        // Tracing
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state)
}
