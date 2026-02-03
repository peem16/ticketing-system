//! GraphQL schema definition
//!
//! Defines Query and Mutation types. Resolvers translate GraphQL operations
//! into gRPC calls to the auth-service via a shared `tonic::Channel`.
//! A circuit breaker protects against cascading failures when the
//! auth-service is unavailable.

use async_graphql::{Context, ErrorExtensions, InputObject, Object, SimpleObject};

use crate::circuit_breaker::CircuitBreaker;
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
// Helpers
// ============================================================================

/// Get a cloned gRPC channel from the GraphQL context.
///
/// The channel is created once at startup and supports multiplexing,
/// so cloning is cheap (just an `Arc` bump).
fn auth_channel(ctx: &Context<'_>) -> async_graphql::Result<tonic::transport::Channel> {
    ctx.data::<tonic::transport::Channel>()
        .map(|ch| ch.clone())
        .map_err(|_| {
            async_graphql::Error::new("Internal configuration error: missing gRPC channel")
                .extend_with(|_, e| e.set("code", "INTERNAL"))
        })
}

/// Get the circuit breaker from the GraphQL context.
fn circuit_breaker(ctx: &Context<'_>) -> async_graphql::Result<CircuitBreaker> {
    ctx.data::<CircuitBreaker>()
        .map(|cb| cb.clone())
        .map_err(|_| {
            async_graphql::Error::new("Internal configuration error: missing circuit breaker")
                .extend_with(|_, e| e.set("code", "INTERNAL"))
        })
}

/// Check the circuit breaker before making a call.
fn check_circuit(cb: &CircuitBreaker) -> async_graphql::Result<()> {
    if !cb.is_available() {
        return Err(
            async_graphql::Error::new("Auth service is temporarily unavailable")
                .extend_with(|_, e| e.set("code", "SERVICE_UNAVAILABLE")),
        );
    }
    Ok(())
}

/// Map a tonic gRPC status to an async-graphql error with appropriate code.
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

        let channel = auth_channel(ctx)?;
        let cb = circuit_breaker(ctx)?;
        check_circuit(&cb)?;

        let mut client = AuthServiceClient::new(channel);
        let result = client
            .get_me(tonic::Request::new(GetMeRequest { token }))
            .await;

        match result {
            Ok(resp) => {
                cb.record_success();
                let resp = resp.into_inner();
                Ok(User {
                    user_id: resp.user_id,
                    email: resp.email,
                    display_name: resp.display_name,
                    is_active: resp.is_active,
                })
            }
            Err(status) => {
                // Only record infrastructure failures, not business errors
                if matches!(
                    status.code(),
                    tonic::Code::Unavailable | tonic::Code::DeadlineExceeded | tonic::Code::Internal
                ) {
                    cb.record_failure();
                }
                Err(grpc_err(status))
            }
        }
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
        let channel = auth_channel(ctx)?;
        let cb = circuit_breaker(ctx)?;
        check_circuit(&cb)?;

        let mut client = AuthServiceClient::new(channel);
        let result = client
            .register(tonic::Request::new(RegisterRequest {
                email: input.email,
                password: input.password,
                display_name: input.display_name,
            }))
            .await;

        match result {
            Ok(resp) => {
                cb.record_success();
                let resp = resp.into_inner();
                Ok(RegisterPayload {
                    user_id: resp.user_id,
                    email: resp.email,
                    display_name: resp.display_name,
                })
            }
            Err(status) => {
                if matches!(
                    status.code(),
                    tonic::Code::Unavailable | tonic::Code::DeadlineExceeded | tonic::Code::Internal
                ) {
                    cb.record_failure();
                }
                Err(grpc_err(status))
            }
        }
    }

    /// Login with email and password, returns a JWT token
    async fn login(
        &self,
        ctx: &Context<'_>,
        input: LoginInput,
    ) -> async_graphql::Result<LoginPayload> {
        let channel = auth_channel(ctx)?;
        let cb = circuit_breaker(ctx)?;
        check_circuit(&cb)?;

        let mut client = AuthServiceClient::new(channel);
        let result = client
            .login(tonic::Request::new(LoginRequest {
                email: input.email,
                password: input.password,
            }))
            .await;

        match result {
            Ok(resp) => {
                cb.record_success();
                let resp = resp.into_inner();
                Ok(LoginPayload {
                    token: resp.token,
                    user_id: resp.user_id,
                    email: resp.email,
                    display_name: resp.display_name,
                })
            }
            Err(status) => {
                if matches!(
                    status.code(),
                    tonic::Code::Unavailable | tonic::Code::DeadlineExceeded | tonic::Code::Internal
                ) {
                    cb.record_failure();
                }
                Err(grpc_err(status))
            }
        }
    }
}

// ============================================================================
// Token wrapper (inserted per-request from Authorization header)
// ============================================================================

/// Bearer token extracted from the HTTP Authorization header
pub struct Token(pub String);
