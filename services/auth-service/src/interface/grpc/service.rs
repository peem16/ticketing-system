//! gRPC service implementation for AuthService

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::application::commands::{
    login_user::{LoginUserCommand, LoginUserUseCase},
    register_user::{RegisterUserCommand, RegisterUserUseCase},
};
use crate::domain::auth::UserRepository;
use crate::domain::error::AuthError;
use crate::infrastructure::db::user_repository_diesel::DieselUserRepository;
use crate::AppState;

pub mod pb {
    tonic::include_proto!("auth");
}

use pb::auth_service_server::AuthService;
use pb::{
    GetMeRequest, GetMeResponse, LoginRequest, LoginResponse, RegisterRequest, RegisterResponse,
    ValidateTokenRequest, ValidateTokenResponse,
};

/// gRPC implementation of the AuthService
pub struct AuthServiceGrpc {
    state: Arc<AppState>,
}

impl AuthServiceGrpc {
    /// Create a new gRPC auth service with shared application state
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

/// Map domain AuthError to gRPC Status
fn map_auth_error(err: AuthError) -> Status {
    match err {
        AuthError::UserAlreadyExists => Status::already_exists(err.to_string()),
        AuthError::UserNotFound => Status::not_found(err.to_string()),
        AuthError::InvalidCredentials => Status::unauthenticated(err.to_string()),
        AuthError::InvalidToken => Status::unauthenticated(err.to_string()),
        AuthError::TokenExpired => Status::unauthenticated(err.to_string()),
        AuthError::InvalidEmail => Status::invalid_argument(err.to_string()),
        AuthError::WeakPassword => Status::invalid_argument(err.to_string()),
        AuthError::AccountInactive => Status::permission_denied(err.to_string()),
        AuthError::Internal(msg) => Status::internal(msg),
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceGrpc {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        let repo = DieselUserRepository::new(self.state.pool.clone());
        let use_case = RegisterUserUseCase::new(&repo, self.state.password_hasher.as_ref());

        let command = RegisterUserCommand {
            email: req.email,
            password: req.password,
            display_name: req.display_name,
        };

        let result = use_case.execute(command).map_err(map_auth_error)?;

        Ok(Response::new(RegisterResponse {
            user_id: result.user_id.to_string(),
            email: result.email,
            display_name: result.display_name,
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let repo = DieselUserRepository::new(self.state.pool.clone());
        let use_case = LoginUserUseCase::new(
            &repo,
            self.state.password_hasher.as_ref(),
            self.state.token_service.as_ref(),
        );

        let command = LoginUserCommand {
            email: req.email,
            password: req.password,
        };

        let result = use_case.execute(command).map_err(map_auth_error)?;

        Ok(Response::new(LoginResponse {
            token: result.token,
            user_id: result.user_id.to_string(),
            email: result.email,
            display_name: result.display_name,
        }))
    }

    async fn get_me(
        &self,
        request: Request<GetMeRequest>,
    ) -> Result<Response<GetMeResponse>, Status> {
        let req = request.into_inner();

        // Validate token
        let token_data = self
            .state
            .token_service
            .validate_token(&req.token)
            .map_err(map_auth_error)?;

        // Fetch user from database
        let repo = DieselUserRepository::new(self.state.pool.clone());
        let user = repo.find_by_id(token_data.user_id).map_err(map_auth_error)?;

        Ok(Response::new(GetMeResponse {
            user_id: user.id().as_uuid().to_string(),
            email: user.email().as_str().to_string(),
            display_name: user.display_name().map(String::from),
            is_active: user.is_active(),
        }))
    }

    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        match self.state.token_service.validate_token(&req.token) {
            Ok(token_data) => Ok(Response::new(ValidateTokenResponse {
                valid: true,
                user_id: token_data.user_id.to_string(),
                email: token_data.email,
            })),
            Err(_) => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                user_id: String::new(),
                email: String::new(),
            })),
        }
    }
}
