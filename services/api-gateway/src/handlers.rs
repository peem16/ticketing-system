//! HTTP request handlers for the API gateway
//!
//! Each handler translates an incoming HTTP request into a gRPC call
//! to the appropriate backend service.

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tonic::Code;

use crate::grpc_client::AuthServiceClient;
use crate::AppState;

// ============================================================================
// Request/Response DTOs
// ============================================================================

/// Request body for user registration
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

/// Request body for user login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Response for successful registration
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
}

/// Response for successful login
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
}

/// Response for current user info
#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub is_active: bool,
}

/// Error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// ============================================================================
// Error handling
// ============================================================================

/// Map gRPC status code to HTTP status code
fn grpc_to_http_status(code: Code) -> StatusCode {
    match code {
        Code::Ok => StatusCode::OK,
        Code::InvalidArgument => StatusCode::BAD_REQUEST,
        Code::NotFound => StatusCode::NOT_FOUND,
        Code::AlreadyExists => StatusCode::CONFLICT,
        Code::PermissionDenied => StatusCode::FORBIDDEN,
        Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Build an error response from a gRPC status
fn grpc_error_response(status: tonic::Status) -> (StatusCode, Json<ErrorResponse>) {
    let http_status = grpc_to_http_status(status.code());
    let body = ErrorResponse {
        error: format!("{:?}", status.code()),
        message: status.message().to_string(),
    };
    (http_status, Json(body))
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /auth/register - Register a new user via auth-service gRPC
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> impl IntoResponse {
    let mut client = match AuthServiceClient::connect(state.auth_service_url.clone()).await {
        Ok(c) => c,
        Err(e) => {
            return grpc_error_response(tonic::Status::unavailable(format!(
                "Auth service unavailable: {}",
                e
            )))
            .into_response();
        }
    };

    let request = tonic::Request::new(crate::grpc_client::RegisterRequest {
        email: body.email,
        password: body.password,
        display_name: body.display_name,
    });

    match client.register(request).await {
        Ok(response) => {
            let resp = response.into_inner();
            let body = RegisterResponse {
                user_id: resp.user_id,
                email: resp.email,
                display_name: resp.display_name,
            };
            (StatusCode::CREATED, Json(body)).into_response()
        }
        Err(status) => grpc_error_response(status).into_response(),
    }
}

/// POST /auth/login - Login via auth-service gRPC
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> impl IntoResponse {
    let mut client = match AuthServiceClient::connect(state.auth_service_url.clone()).await {
        Ok(c) => c,
        Err(e) => {
            return grpc_error_response(tonic::Status::unavailable(format!(
                "Auth service unavailable: {}",
                e
            )))
            .into_response();
        }
    };

    let request = tonic::Request::new(crate::grpc_client::LoginRequest {
        email: body.email,
        password: body.password,
    });

    match client.login(request).await {
        Ok(response) => {
            let resp = response.into_inner();
            let body = LoginResponse {
                token: resp.token,
                user_id: resp.user_id,
                email: resp.email,
                display_name: resp.display_name,
            };
            Json(body).into_response()
        }
        Err(status) => grpc_error_response(status).into_response(),
    }
}

/// GET /auth/me - Get current user info via auth-service gRPC
pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // Extract token from Authorization header
    let token = match headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => {
            let body = ErrorResponse {
                error: "invalid_token".to_string(),
                message: "Missing or invalid Authorization header".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, Json(body)).into_response();
        }
    };

    let mut client = match AuthServiceClient::connect(state.auth_service_url.clone()).await {
        Ok(c) => c,
        Err(e) => {
            return grpc_error_response(tonic::Status::unavailable(format!(
                "Auth service unavailable: {}",
                e
            )))
            .into_response();
        }
    };

    let request = tonic::Request::new(crate::grpc_client::GetMeRequest { token });

    match client.get_me(request).await {
        Ok(response) => {
            let resp = response.into_inner();
            let body = MeResponse {
                user_id: resp.user_id,
                email: resp.email,
                display_name: resp.display_name,
                is_active: resp.is_active,
            };
            Json(body).into_response()
        }
        Err(status) => grpc_error_response(status).into_response(),
    }
}

/// GET /health - Gateway health check
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "api-gateway"
    }))
}
