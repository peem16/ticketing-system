//! Router configuration for the API gateway
//!
//! Exposes a single `/graphql` POST endpoint and an interactive GraphiQL
//! playground at `GET /graphql` for development.
//! Includes security headers, request body limits, and rate limiting.

use std::num::NonZeroU32;
use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

use crate::grpc_client::AuthServiceClient;
use crate::schema::Token;
use crate::AppState;

/// Shared rate limiter type
type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Handle incoming GraphQL requests
async fn graphql_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    // Extract Bearer token from Authorization header (if present)
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or_default()
        .to_string();

    let request = req.into_inner().data(Token(token));
    state.schema.execute(request).await.into()
}

/// Serve the GraphiQL interactive playground
async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

/// Health check endpoint (REST for infrastructure probes).
///
/// Checks connectivity to the auth-service via gRPC.
async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let auth_ok = check_auth_service_health(&state.grpc_channel).await;

    let status = if auth_ok {
        StatusCode::OK
    } else {
        StatusCode::OK // Gateway itself is healthy, downstream may not be
    };

    (
        status,
        axum::Json(serde_json::json!({
            "status": "healthy",
            "service": "api-gateway",
            "checks": {
                "auth_service": if auth_ok { "reachable" } else { "unreachable" }
            }
        })),
    )
}

/// Probe the auth-service gRPC endpoint with a ValidateToken call.
async fn check_auth_service_health(channel: &tonic::transport::Channel) -> bool {
    use crate::grpc_client::ValidateTokenRequest;

    let mut client = AuthServiceClient::new(channel.clone());
    // Send a no-op validation (empty token will return valid=false but proves connectivity)
    client
        .validate_token(tonic::Request::new(ValidateTokenRequest {
            token: String::new(),
        }))
        .await
        .is_ok()
}

/// Rate-limiting middleware
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

/// Create the gateway router with GraphQL, health endpoints, and security layers
pub fn create_router(state: AppState, rate_limit_per_second: u32) -> Router {
    let mut router = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/health", get(health));

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

    // CORS — permissive by default; tighten for production
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    router
        // Request body size limit (2 MB — GraphQL queries can be verbose)
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
        // CORS
        .layer(cors)
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
        // Tracing
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state)
}
