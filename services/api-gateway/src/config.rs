//! API Gateway configuration loaded from environment variables

use std::env;

/// Configuration for the API gateway
#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub auth_service_url: String,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Errors
    /// Returns error if required environment variables are missing or invalid
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("SERVER_PORT"))?;

        let auth_service_url = env::var("AUTH_SERVICE_GRPC_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

        Ok(Self {
            server_host,
            server_port,
            auth_service_url,
        })
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid value for environment variable: {0}")]
    InvalidValue(&'static str),
}
