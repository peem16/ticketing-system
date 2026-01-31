//! JWT token service implementation

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::auth::{TokenData, TokenService};
use crate::domain::error::AuthError;
use crate::domain::user::User;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Subject (user ID)
    sub: String,
    /// User email
    email: String,
    /// Issued at timestamp
    iat: i64,
    /// Expiration timestamp
    exp: i64,
}

/// JWT-based implementation of the TokenService trait
pub struct JwtTokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration_secs: i64,
}

impl JwtTokenService {
    /// Create a new JWT token service
    ///
    /// # Arguments
    /// * `secret` - The secret key for signing tokens
    /// * `expiration_secs` - Token expiration time in seconds
    #[must_use]
    pub fn new(secret: String, expiration_secs: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiration_secs,
        }
    }
}

impl TokenService for JwtTokenService {
    fn create_token(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiration = now + Duration::seconds(self.expiration_secs);

        let claims = Claims {
            sub: user.id().as_uuid().to_string(),
            email: user.email().as_str().to_string(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::Internal(format!("Failed to create token: {}", e)))
    }

    fn validate_token(&self, token: &str) -> Result<TokenData, AuthError> {
        let validation = Validation::default();

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation).map_err(|e| {
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            }
        })?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(TokenData {
            user_id,
            email: token_data.claims.email,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::{Email, HashedPassword};

    fn create_test_user() -> User {
        let email = Email::new("test@example.com").unwrap();
        let password = HashedPassword::from_hash("hashed".to_string());
        User::new(email, password, Some("Test User".to_string()))
    }

    #[test]
    fn test_create_and_validate_token() {
        let service = JwtTokenService::new("test-secret-key".to_string(), 3600);
        let user = create_test_user();

        let token = service.create_token(&user).unwrap();
        let data = service.validate_token(&token).unwrap();

        assert_eq!(data.user_id, user.id().as_uuid());
        assert_eq!(data.email, "test@example.com");
    }

    #[test]
    fn test_invalid_token() {
        let service = JwtTokenService::new("test-secret-key".to_string(), 3600);

        let result = service.validate_token("invalid.token.here");
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_expired_token() {
        // Create a service with 0 second expiration (immediate expiry)
        let service = JwtTokenService::new("test-secret-key".to_string(), -1);
        let user = create_test_user();

        let token = service.create_token(&user).unwrap();
        let result = service.validate_token(&token);

        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[test]
    fn test_wrong_secret() {
        let service1 = JwtTokenService::new("secret-1".to_string(), 3600);
        let service2 = JwtTokenService::new("secret-2".to_string(), 3600);
        let user = create_test_user();

        let token = service1.create_token(&user).unwrap();
        let result = service2.validate_token(&token);

        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }
}
