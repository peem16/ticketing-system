//! Database seeding utilities
//!
//! Provides functions to seed initial data into the database.

use tracing::info;

use crate::domain::auth::{PasswordHasher, UserRepository};
use crate::domain::user::{Email, User};

/// Seed data configuration
pub struct SeedConfig {
    pub admin_email: String,
    pub admin_password: String,
    pub admin_display_name: String,
}

impl Default for SeedConfig {
    fn default() -> Self {
        Self {
            admin_email: "admin@example.com".to_string(),
            admin_password: "Admin123!".to_string(),
            admin_display_name: "System Administrator".to_string(),
        }
    }
}

/// Seed initial users into the database
///
/// Creates an admin user if one doesn't exist.
///
/// # Errors
/// Returns error if seeding fails
pub fn seed_users<R, H>(
    repo: &R,
    hasher: &H,
    config: &SeedConfig,
) -> Result<(), crate::domain::error::AuthError>
where
    R: UserRepository,
    H: PasswordHasher,
{
    // Check if admin user already exists
    if repo.exists_by_email(&config.admin_email)? {
        info!("Admin user already exists, skipping seed");
        return Ok(());
    }

    // Create admin user
    let email = Email::new(&config.admin_email)?;
    let hashed_password = hasher.hash(&config.admin_password)?;
    let admin_user = User::new(email, hashed_password, Some(config.admin_display_name.clone()));

    repo.create(&admin_user)?;
    info!("Created admin user: {}", config.admin_email);

    Ok(())
}
