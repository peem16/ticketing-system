//! HTTP request handlers for auth endpoints

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::application::commands::{
    login_user::{LoginUserCommand, LoginUserUseCase},
    register_user::{RegisterUserCommand, RegisterUserUseCase},
};
use crate::domain::auth::UserRepository;
use crate::domain::error::AuthError;
use crate::infrastructure::db::user_repository_diesel::DieselUserRepository;
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

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// ============================================================================
// Error handling
// ============================================================================

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_type) = match &self {
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "user_exists"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "user_not_found"),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "token_expired"),
            AuthError::InvalidEmail => (StatusCode::BAD_REQUEST, "invalid_email"),
            AuthError::WeakPassword => (StatusCode::BAD_REQUEST, "weak_password"),
            AuthError::AccountInactive => (StatusCode::FORBIDDEN, "account_inactive"),
            AuthError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /auth/register - Register a new user
///
/// Diesel and Argon2 operations are wrapped in `spawn_blocking` to avoid
/// blocking the Tokio async runtime.
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let result = tokio::task::spawn_blocking(move || {
        let repo = DieselUserRepository::new(state.pool.clone());
        let use_case = RegisterUserUseCase::new(&repo, state.password_hasher.as_ref());

        let command = RegisterUserCommand {
            email: body.email,
            password: body.password,
            display_name: body.display_name,
        };

        use_case.execute(command)
    })
    .await
    .map_err(|e| AuthError::Internal(format!("Task join error: {}", e)))??;

    let response = RegisterResponse {
        user_id: result.user_id.to_string(),
        email: result.email,
        display_name: result.display_name,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// POST /auth/login - Authenticate user and return JWT
///
/// Diesel and Argon2 operations are wrapped in `spawn_blocking`.
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let result = tokio::task::spawn_blocking(move || {
        let repo = DieselUserRepository::new(state.pool.clone());
        let use_case = LoginUserUseCase::new(
            &repo,
            state.password_hasher.as_ref(),
            state.token_service.as_ref(),
        );

        let command = LoginUserCommand {
            email: body.email,
            password: body.password,
        };

        use_case.execute(command)
    })
    .await
    .map_err(|e| AuthError::Internal(format!("Task join error: {}", e)))??;

    let response = LoginResponse {
        token: result.token,
        user_id: result.user_id.to_string(),
        email: result.email,
        display_name: result.display_name,
    };

    Ok(Json(response))
}

/// GET /auth/me - Get current user info from JWT
///
/// Diesel operations are wrapped in `spawn_blocking`.
pub async fn me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AuthError> {
    // Extract token from Authorization header
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::InvalidToken)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidToken)?
        .to_string();

    let result = tokio::task::spawn_blocking(move || {
        // Validate token (may hit moka cache)
        let token_data = state.token_service.validate_token(&token)?;

        // Get user from database
        let repo = DieselUserRepository::new(state.pool.clone());
        let user = repo.find_by_id(token_data.user_id)?;

        Ok::<MeResponse, AuthError>(MeResponse {
            user_id: user.id().as_uuid().to_string(),
            email: user.email().as_str().to_string(),
            display_name: user.display_name().map(String::from),
            is_active: user.is_active(),
        })
    })
    .await
    .map_err(|e| AuthError::Internal(format!("Task join error: {}", e)))??;

    Ok(Json(result))
}

/// GET /health - Health check endpoint
///
/// Verifies database connectivity by attempting to acquire a pool connection.
pub async fn health(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let db_ok = tokio::task::spawn_blocking(move || state.pool.get().is_ok())
        .await
        .unwrap_or(false);

    let status = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(serde_json::json!({
            "status": if db_ok { "healthy" } else { "unhealthy" },
            "service": "auth-service",
            "checks": {
                "database": if db_ok { "connected" } else { "disconnected" }
            }
        })),
    )
}
