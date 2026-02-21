//! gRPC client for communicating with the auth-service

pub mod pb {
    tonic::include_proto!("auth");
}

pub use pb::auth_service_client::AuthServiceClient;
pub use pb::{
    GetMeRequest, LoginRequest, RegisterRequest, ValidateTokenRequest,
};
