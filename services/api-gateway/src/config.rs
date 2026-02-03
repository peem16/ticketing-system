//! API Gateway configuration loaded from environment variables

use std::env;

/// Configuration for the API gateway
#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub auth_service_url: String,
    /// Log format: "json" for structured JSON, anything else for human-readable
    pub log_format: String,
    /// Global rate limit: max requests per second (0 = disabled)
    pub rate_limit_per_second: u32,
    /// gRPC call timeout in seconds
    pub grpc_timeout_secs: u64,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker recovery timeout in seconds
    pub circuit_breaker_recovery_secs: u64,
    /// GraphQL max query depth
    pub graphql_max_depth: usize,
    /// GraphQL max query complexity
    pub graphql_max_complexity: usize,
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

        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

        let rate_limit_per_second = env::var("RATE_LIMIT_PER_SECOND")
            .unwrap_or_else(|_| "200".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("RATE_LIMIT_PER_SECOND"))?;

        let grpc_timeout_secs = env::var("GRPC_TIMEOUT_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("GRPC_TIMEOUT_SECS"))?;

        let circuit_breaker_threshold = env::var("CIRCUIT_BREAKER_THRESHOLD")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("CIRCUIT_BREAKER_THRESHOLD"))?;

        let circuit_breaker_recovery_secs = env::var("CIRCUIT_BREAKER_RECOVERY_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("CIRCUIT_BREAKER_RECOVERY_SECS"))?;

        let graphql_max_depth = env::var("GRAPHQL_MAX_DEPTH")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("GRAPHQL_MAX_DEPTH"))?;

        let graphql_max_complexity = env::var("GRAPHQL_MAX_COMPLEXITY")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("GRAPHQL_MAX_COMPLEXITY"))?;

        Ok(Self {
            server_host,
            server_port,
            auth_service_url,
            log_format,
            rate_limit_per_second,
            grpc_timeout_secs,
            circuit_breaker_threshold,
            circuit_breaker_recovery_secs,
            graphql_max_depth,
            graphql_max_complexity,
        })
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid value for environment variable: {0}")]
    InvalidValue(&'static str),
}
