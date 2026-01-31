//! Domain layer errors
//!
//! These errors represent business rule violations and domain-level failures.
//! They are independent of infrastructure concerns.

use std::fmt;

/// Domain-level authentication errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// User with the given email already exists
    UserAlreadyExists,

    /// User was not found
    UserNotFound,

    /// Invalid credentials provided
    InvalidCredentials,

    /// Token is invalid or expired
    InvalidToken,

    /// Token has expired
    TokenExpired,

    /// Email format is invalid
    InvalidEmail,

    /// Password does not meet requirements
    WeakPassword,

    /// User account is inactive
    AccountInactive,

    /// Internal error during operation
    Internal(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserAlreadyExists => write!(f, "User with this email already exists"),
            Self::UserNotFound => write!(f, "User not found"),
            Self::InvalidCredentials => write!(f, "Invalid email or password"),
            Self::InvalidToken => write!(f, "Invalid or malformed token"),
            Self::TokenExpired => write!(f, "Token has expired"),
            Self::InvalidEmail => write!(f, "Invalid email format"),
            Self::WeakPassword => write!(f, "Password does not meet minimum requirements"),
            Self::AccountInactive => write!(f, "User account is inactive"),
            Self::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}
