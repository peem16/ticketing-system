//! User domain entity and value objects
//!
//! Contains the core User aggregate and associated value types.
//! No infrastructure dependencies allowed in this module.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::AuthError;

/// Unique identifier for a user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);

impl UserId {
    /// Create a new random user ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a UserId from an existing UUID
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID value
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

/// Email value object with basic validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    /// Create a new Email after basic validation
    ///
    /// # Errors
    /// Returns `AuthError::InvalidEmail` if format is invalid
    pub fn new(value: &str) -> Result<Self, AuthError> {
        let trimmed = value.trim().to_lowercase();

        // Basic email validation
        if trimmed.is_empty() {
            return Err(AuthError::InvalidEmail);
        }

        if !trimmed.contains('@') {
            return Err(AuthError::InvalidEmail);
        }

        let parts: Vec<&str> = trimmed.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(AuthError::InvalidEmail);
        }

        if !parts[1].contains('.') {
            return Err(AuthError::InvalidEmail);
        }

        Ok(Self(trimmed))
    }

    /// Get the email as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Hashed password value object
///
/// This type ensures passwords are never stored in plain text in the domain.
#[derive(Debug, Clone)]
pub struct HashedPassword(String);

impl HashedPassword {
    /// Create a HashedPassword from an already hashed string
    ///
    /// This should only be called with properly hashed values from the PasswordHasher.
    #[must_use]
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    /// Get the hash as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// User domain entity
#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: Email,
    hashed_password: HashedPassword,
    display_name: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user
    ///
    /// # Arguments
    /// * `email` - Valid email address
    /// * `hashed_password` - Already hashed password
    /// * `display_name` - Optional display name
    #[must_use]
    pub fn new(email: Email, hashed_password: HashedPassword, display_name: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::new(),
            email,
            hashed_password,
            display_name,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct a user from persistence
    ///
    /// Used when loading from database - bypasses normal construction rules.
    #[must_use]
    pub fn from_persistence(
        id: Uuid,
        email: String,
        hashed_password: String,
        display_name: Option<String>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: UserId::from_uuid(id),
            // Trust persisted email is valid
            email: Email(email),
            hashed_password: HashedPassword(hashed_password),
            display_name,
            is_active,
            created_at,
            updated_at,
        }
    }

    /// Get the user's ID
    #[must_use]
    pub fn id(&self) -> UserId {
        self.id
    }

    /// Get the user's email
    #[must_use]
    pub fn email(&self) -> &Email {
        &self.email
    }

    /// Get the user's hashed password
    #[must_use]
    pub fn hashed_password(&self) -> &HashedPassword {
        &self.hashed_password
    }

    /// Get the user's display name
    #[must_use]
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    /// Check if the user account is active
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get the creation timestamp
    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get the last update timestamp
    #[must_use]
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Deactivate the user account
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Activate the user account
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_valid() {
        assert!(Email::new("test@example.com").is_ok());
        assert!(Email::new("user.name@domain.co.uk").is_ok());
    }

    #[test]
    fn test_email_invalid() {
        assert!(Email::new("").is_err());
        assert!(Email::new("invalid").is_err());
        assert!(Email::new("@example.com").is_err());
        assert!(Email::new("test@").is_err());
        assert!(Email::new("test@domain").is_err());
    }

    #[test]
    fn test_email_normalized() {
        let email = Email::new("  TEST@EXAMPLE.COM  ").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn test_user_creation() {
        let email = Email::new("test@example.com").unwrap();
        let password = HashedPassword::from_hash("hashed".to_string());
        let user = User::new(email, password, Some("Test User".to_string()));

        assert!(user.is_active());
        assert_eq!(user.email().as_str(), "test@example.com");
        assert_eq!(user.display_name(), Some("Test User"));
    }
}
