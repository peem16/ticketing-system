//! Cached wrapper around TokenService for token validation
//!
//! Uses moka in-memory cache to avoid repeated JWT decode + DB lookups
//! for the same token within a short TTL window.

use std::sync::Arc;
use std::time::Duration;

use moka::sync::Cache;

use crate::domain::auth::{TokenData, TokenService};
use crate::domain::error::AuthError;
use crate::domain::user::User;

/// A caching decorator over any `TokenService` implementation.
///
/// `create_token` delegates directly; `validate_token` results are cached
/// by token string with a configurable TTL.
pub struct CachedTokenService {
    inner: Arc<dyn TokenService + Send + Sync>,
    cache: Cache<String, TokenData>,
}

impl CachedTokenService {
    /// Create a new cached token service.
    ///
    /// # Arguments
    /// * `inner` - The underlying token service to delegate to
    /// * `ttl_secs` - Time-to-live for cached validation results in seconds
    /// * `max_capacity` - Maximum number of cached entries
    pub fn new(
        inner: Arc<dyn TokenService + Send + Sync>,
        ttl_secs: u64,
        max_capacity: u64,
    ) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();

        Self { inner, cache }
    }
}

impl TokenService for CachedTokenService {
    fn create_token(&self, user: &User) -> Result<String, AuthError> {
        self.inner.create_token(user)
    }

    fn validate_token(&self, token: &str) -> Result<TokenData, AuthError> {
        // Check cache first
        if let Some(data) = self.cache.get(&token.to_string()) {
            return Ok(data);
        }

        // Cache miss — validate via inner service
        let data = self.inner.validate_token(token)?;
        self.cache.insert(token.to_string(), data.clone());
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::{Email, HashedPassword};
    use uuid::Uuid;

    struct FakeTokenService;

    impl TokenService for FakeTokenService {
        fn create_token(&self, _user: &User) -> Result<String, AuthError> {
            Ok("fake_token".to_string())
        }

        fn validate_token(&self, token: &str) -> Result<TokenData, AuthError> {
            if token == "valid" {
                Ok(TokenData {
                    user_id: Uuid::nil(),
                    email: "cached@example.com".to_string(),
                })
            } else {
                Err(AuthError::InvalidToken)
            }
        }
    }

    #[test]
    fn test_cached_validation_returns_same_result() {
        let inner = Arc::new(FakeTokenService);
        let cached = CachedTokenService::new(inner, 60, 100);

        let first = cached.validate_token("valid").unwrap();
        let second = cached.validate_token("valid").unwrap();

        assert_eq!(first.user_id, second.user_id);
        assert_eq!(first.email, second.email);
    }

    #[test]
    fn test_cached_invalid_token_not_cached() {
        let inner = Arc::new(FakeTokenService);
        let cached = CachedTokenService::new(inner, 60, 100);

        assert!(cached.validate_token("invalid").is_err());
        // Errors are not cached, so this should also hit the inner service
        assert!(cached.validate_token("invalid").is_err());
    }
}
