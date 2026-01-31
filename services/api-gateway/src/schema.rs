//! GraphQL schema definition
//!
//! Defines Query and Mutation types. Resolvers translate GraphQL operations
//! into gRPC calls to the auth-service.

use async_graphql::{Context, ErrorExtensions, Object, SimpleObject, InputObject};

use crate::grpc_client::{AuthServiceClient, GetMeRequest, LoginRequest, RegisterRequest};

// ============================================================================
// GraphQL types
// ============================================================================

/// Registered user returned after sign-up
#[derive(SimpleObject)]
pub struct RegisterPayload {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
}

/// Login result containing a JWT token and user details
#[derive(SimpleObject)]
pub struct LoginPayload {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
}

/// Current authenticated user info
#[derive(SimpleObject)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub is_active: bool,
}

// ============================================================================
// Input types
// ============================================================================

/// Input for user registration
#[derive(InputObject)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

/// Input for user login
#[derive(InputObject)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

// ============================================================================
// Helper
// ============================================================================

/// Connect to the auth-service gRPC endpoint stored in context
async fn auth_client(ctx: &Context<'_>) -> async_graphql::Result<AuthServiceClient<tonic::transport::Channel>> {
    let url = ctx.data::<String>().map_err(|_| {
        async_graphql::Error::new("Internal configuration error")
            .extend_with(|_, e| e.set("code", "INTERNAL"))
    })?;

    AuthServiceClient::connect(url.clone()).await.map_err(|e| {
        async_graphql::Error::new(format!("Auth service unavailable: {}", e))
            .extend_with(|_, e| e.set("code", "SERVICE_UNAVAILABLE"))
    })
}

/// Map a tonic gRPC status to an async-graphql error with appropriate code
fn grpc_err(status: tonic::Status) -> async_graphql::Error {
    let code = match status.code() {
        tonic::Code::InvalidArgument => "BAD_USER_INPUT",
        tonic::Code::NotFound => "NOT_FOUND",
        tonic::Code::AlreadyExists => "ALREADY_EXISTS",
        tonic::Code::PermissionDenied => "FORBIDDEN",
        tonic::Code::Unauthenticated => "UNAUTHENTICATED",
        tonic::Code::Unavailable => "SERVICE_UNAVAILABLE",
        _ => "INTERNAL_SERVER_ERROR",
    };
    async_graphql::Error::new(status.message().to_string())
        .extend_with(|_, e| e.set("code", code))
}

// ============================================================================
// Query
// ============================================================================

/// GraphQL query root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get the currently authenticated user. Requires `Authorization: Bearer <token>` header.
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let token = ctx
            .data_opt::<Token>()
            .map(|t| t.0.clone())
            .unwrap_or_default();

        if token.is_empty() {
            return Err(
                async_graphql::Error::new("Missing or invalid Authorization header")
                    .extend_with(|_, e| e.set("code", "UNAUTHENTICATED")),
            );
        }

        let mut client = auth_client(ctx).await?;
        let resp = client
            .get_me(tonic::Request::new(GetMeRequest { token }))
            .await
            .map_err(grpc_err)?
            .into_inner();

        Ok(User {
            user_id: resp.user_id,
            email: resp.email,
            display_name: resp.display_name,
            is_active: resp.is_active,
        })
    }

    /// Gateway health check
    async fn health(&self) -> &str {
        "ok"
    }
}

// ============================================================================
// Mutation
// ============================================================================

/// GraphQL mutation root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Register a new user account
    async fn register(
        &self,
        ctx: &Context<'_>,
        input: RegisterInput,
    ) -> async_graphql::Result<RegisterPayload> {
        let mut client = auth_client(ctx).await?;

        let resp = client
            .register(tonic::Request::new(RegisterRequest {
                email: input.email,
                password: input.password,
                display_name: input.display_name,
            }))
            .await
            .map_err(grpc_err)?
            .into_inner();

        Ok(RegisterPayload {
            user_id: resp.user_id,
            email: resp.email,
            display_name: resp.display_name,
        })
    }

    /// Login with email and password, returns a JWT token
    async fn login(
        &self,
        ctx: &Context<'_>,
        input: LoginInput,
    ) -> async_graphql::Result<LoginPayload> {
        let mut client = auth_client(ctx).await?;

        let resp = client
            .login(tonic::Request::new(LoginRequest {
                email: input.email,
                password: input.password,
            }))
            .await
            .map_err(grpc_err)?
            .into_inner();

        Ok(LoginPayload {
            token: resp.token,
            user_id: resp.user_id,
            email: resp.email,
            display_name: resp.display_name,
        })
    }
}

// ============================================================================
// Token wrapper (inserted per-request from Authorization header)
// ============================================================================

/// Bearer token extracted from the HTTP Authorization header
pub struct Token(pub String);
