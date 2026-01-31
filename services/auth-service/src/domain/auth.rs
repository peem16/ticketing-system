//! Authentication domain interfaces
//!
//! Defines traits for authentication-related services.
//! Implementations are in the infrastructure layer.

use uuid::Uuid;

use super::error::AuthError;
use super::user::{HashedPassword, User};

/// Repository interface for user persistence
pub trait UserRepository {
    /// Find a user by their ID
    ///
    /// # Errors
    /// Returns `AuthError::UserNotFound` if user doesn't exist
    /// Returns `AuthError::Internal` on database errors
    fn find_by_id(&self, id: Uuid) -> Result<User, AuthError>;

    /// Find a user by their email address
    ///
    /// # Errors
    /// Returns `AuthError::UserNotFound` if user doesn't exist
    /// Returns `AuthError::Internal` on database errors
    fn find_by_email(&self, email: &str) -> Result<User, AuthError>;

    /// Check if a user with the given email exists
    fn exists_by_email(&self, email: &str) -> Result<bool, AuthError>;

    /// Create a new user
    ///
    /// # Errors
    /// Returns `AuthError::UserAlreadyExists` if email is taken
    /// Returns `AuthError::Internal` on database errors
    fn create(&self, user: &User) -> Result<User, AuthError>;

    /// Update an existing user
    ///
    /// # Errors
    /// Returns `AuthError::UserNotFound` if user doesn't exist
    /// Returns `AuthError::Internal` on database errors
    fn update(&self, user: &User) -> Result<User, AuthError>;
}

/// Service interface for password hashing
pub trait PasswordHasher {
    /// Hash a plain text password
    ///
    /// # Errors
    /// Returns `AuthError::Internal` if hashing fails
    fn hash(&self, plain_password: &str) -> Result<HashedPassword, AuthError>;

    /// Verify a plain text password against a hash
    ///
    /// # Returns
    /// `Ok(true)` if password matches
    /// `Ok(false)` if password doesn't match
    ///
    /// # Errors
    /// Returns `AuthError::Internal` if verification fails unexpectedly
    fn verify(&self, plain_password: &str, hashed: &HashedPassword) -> Result<bool, AuthError>;
}

/// Data extracted from a validated token
#[derive(Debug, Clone)]
pub struct TokenData {
    pub user_id: Uuid,
    pub email: String,
}

/// Service interface for JWT token operations
pub trait TokenService {
    /// Create a new JWT token for a user
    ///
    /// # Errors
    /// Returns `AuthError::Internal` if token creation fails
    fn create_token(&self, user: &User) -> Result<String, AuthError>;

    /// Validate a token and extract its data
    ///
    /// # Errors
    /// Returns `AuthError::InvalidToken` if token is malformed
    /// Returns `AuthError::TokenExpired` if token has expired
    fn validate_token(&self, token: &str) -> Result<TokenData, AuthError>;
}
