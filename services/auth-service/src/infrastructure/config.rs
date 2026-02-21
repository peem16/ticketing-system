//! Application configuration loaded from environment variables

use std::env;

/// Configuration for the auth service
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_secs: i64,
    pub server_host: String,
    pub server_port: u16,
    pub grpc_port: u16,
    /// Maximum number of connections in the database pool
    pub db_pool_max_size: u32,
    /// Token validation cache TTL in seconds (0 = disabled)
    pub token_cache_ttl_secs: u64,
    /// Token validation cache max capacity
    pub token_cache_max_capacity: u64,
    /// Log format: "json" for structured JSON, anything else for human-readable
    pub log_format: String,
    /// Global rate limit: max requests per second (0 = disabled)
    pub rate_limit_per_second: u32,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Errors
    /// Returns error if required environment variables are missing
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnv("DATABASE_URL"))?;

        let jwt_secret = env::var("AUTH_JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnv("AUTH_JWT_SECRET"))?;

        let jwt_expiration_secs = env::var("AUTH_JWT_EXP_SECS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("AUTH_JWT_EXP_SECS"))?;

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("SERVER_PORT"))?;

        let grpc_port = env::var("GRPC_PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("GRPC_PORT"))?;

        let db_pool_max_size = env::var("DB_POOL_MAX_SIZE")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("DB_POOL_MAX_SIZE"))?;

        let token_cache_ttl_secs = env::var("TOKEN_CACHE_TTL_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("TOKEN_CACHE_TTL_SECS"))?;

        let token_cache_max_capacity = env::var("TOKEN_CACHE_MAX_CAPACITY")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("TOKEN_CACHE_MAX_CAPACITY"))?;

        let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

        let rate_limit_per_second = env::var("RATE_LIMIT_PER_SECOND")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("RATE_LIMIT_PER_SECOND"))?;

        Ok(Self {
            database_url,
            jwt_secret,
            jwt_expiration_secs,
            server_host,
            server_port,
            grpc_port,
            db_pool_max_size,
            token_cache_ttl_secs,
            token_cache_max_capacity,
            log_format,
            rate_limit_per_second,
        })
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnv(&'static str),

    #[error("Invalid value for environment variable: {0}")]
    InvalidValue(&'static str),
}
