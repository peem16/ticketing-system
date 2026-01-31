//! Router configuration for the API gateway
//!
//! Exposes a single `/graphql` POST endpoint and an interactive GraphiQL
//! playground at `GET /graphql` for development.

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::HeaderMap,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::trace::TraceLayer;

use crate::schema::Token;
use crate::AppState;

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

/// Health check endpoint (remains REST for infrastructure probes)
async fn health() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "api-gateway"
    }))
}

/// Create the gateway router with GraphQL and health endpoints
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
