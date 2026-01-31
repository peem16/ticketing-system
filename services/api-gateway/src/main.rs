//! API Gateway
//!
//! GraphQL entry point that routes operations to backend gRPC microservices.
//! Uses `async-graphql` for the schema and `tonic` as the gRPC client.

mod config;
mod grpc_client;
mod router;
mod schema;

use async_graphql::Schema;
use tokio::net::TcpListener;
use tracing::info;

use crate::config::Config;
use crate::schema::{MutationRoot, QueryRoot};

/// Type alias for the gateway GraphQL schema
pub type GatewaySchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;

/// Shared application state passed to all handlers
#[derive(Clone)]
pub struct AppState {
    pub schema: GatewaySchema,
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

    // Build GraphQL schema with the auth-service URL injected as global data
    let schema = Schema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(config.auth_service_url.clone())
        .finish();

    // Build application state
    let state = AppState { schema };

    // Build router
    let app = router::create_router(state);

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    info!("API Gateway listening on {}", addr);
    info!("GraphiQL playground: http://{}/graphql", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
