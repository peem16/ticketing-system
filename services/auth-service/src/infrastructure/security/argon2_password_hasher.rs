//! Argon2-based password hashing implementation

use argon2::{
    password_hash::{PasswordHash, PasswordHasher as Argon2Hasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

use crate::domain::auth::PasswordHasher;
use crate::domain::error::AuthError;
use crate::domain::user::HashedPassword;

/// Argon2 implementation of the PasswordHasher trait
pub struct Argon2PasswordHasher {
    argon2: Argon2<'static>,
}

impl Argon2PasswordHasher {
    /// Create a new Argon2 password hasher with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, plain_password: &str) -> Result<HashedPassword, AuthError> {
        let salt = SaltString::generate(&mut OsRng);

        let hash = self
            .argon2
            .hash_password(plain_password.as_bytes(), &salt)
            .map_err(|e| AuthError::Internal(format!("Failed to hash password: {}", e)))?;

        Ok(HashedPassword::from_hash(hash.to_string()))
    }

    fn verify(&self, plain_password: &str, hashed: &HashedPassword) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hashed.as_str())
            .map_err(|e| AuthError::Internal(format!("Invalid password hash format: {}", e)))?;

        match self.argon2.verify_password(plain_password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthError::Internal(format!("Password verification error: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let hasher = Argon2PasswordHasher::new();
        let password = "SecurePassword123!";

        let hashed = hasher.hash(password).unwrap();
        
        assert!(hasher.verify(password, &hashed).unwrap());
        assert!(!hasher.verify("WrongPassword", &hashed).unwrap());
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let hasher = Argon2PasswordHasher::new();
        let password = "TestPassword";

        let hash1 = hasher.hash(password).unwrap();
        let hash2 = hasher.hash(password).unwrap();

        // Hashes should be different due to different salts
        assert_ne!(hash1.as_str(), hash2.as_str());

        // But both should verify correctly
        assert!(hasher.verify(password, &hash1).unwrap());
        assert!(hasher.verify(password, &hash2).unwrap());
    }
}
