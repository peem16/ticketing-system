//! Diesel implementation of the UserRepository trait

use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::auth::UserRepository;
use crate::domain::error::AuthError;
use crate::domain::user::User;

use super::connection::DbPool;
use super::models::{DbUser, NewDbUser};
use super::schema::users;

/// Diesel-based implementation of UserRepository
pub struct DieselUserRepository {
    pool: DbPool,
}

impl DieselUserRepository {
    /// Create a new repository instance
    #[must_use]
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get a connection from the pool
    fn conn(&self) -> Result<super::connection::PooledDbConnection, AuthError> {
        self.pool
            .get()
            .map_err(|e| AuthError::Internal(format!("Failed to get database connection: {}", e)))
    }
}

impl UserRepository for DieselUserRepository {
    fn find_by_id(&self, id: Uuid) -> Result<User, AuthError> {
        let mut conn = self.conn()?;

        let db_user: DbUser = users::table
            .filter(users::id.eq(id))
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AuthError::UserNotFound,
                _ => AuthError::Internal(format!("Database error: {}", e)),
            })?;

        Ok(db_user_to_domain(db_user))
    }

    fn find_by_email(&self, email: &str) -> Result<User, AuthError> {
        let mut conn = self.conn()?;
        let normalized_email = email.trim().to_lowercase();

        let db_user: DbUser = users::table
            .filter(users::email.eq(&normalized_email))
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AuthError::UserNotFound,
                _ => AuthError::Internal(format!("Database error: {}", e)),
            })?;

        Ok(db_user_to_domain(db_user))
    }

    fn exists_by_email(&self, email: &str) -> Result<bool, AuthError> {
        let mut conn = self.conn()?;
        let normalized_email = email.trim().to_lowercase();

        let count: i64 = users::table
            .filter(users::email.eq(&normalized_email))
            .count()
            .get_result(&mut conn)
            .map_err(|e| AuthError::Internal(format!("Database error: {}", e)))?;

        Ok(count > 0)
    }

    fn create(&self, user: &User) -> Result<User, AuthError> {
        let mut conn = self.conn()?;

        let new_user = NewDbUser {
            id: user.id().as_uuid(),
            email: user.email().as_str(),
            hashed_password: user.hashed_password().as_str(),
            display_name: user.display_name(),
            is_active: user.is_active(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut conn)
            .map_err(|e| {
                if let diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) = e
                {
                    AuthError::UserAlreadyExists
                } else {
                    AuthError::Internal(format!("Failed to create user: {}", e))
                }
            })?;

        // Return the user as created
        self.find_by_id(user.id().as_uuid())
    }

    fn update(&self, user: &User) -> Result<User, AuthError> {
        let mut conn = self.conn()?;

        let updated_rows = diesel::update(users::table.filter(users::id.eq(user.id().as_uuid())))
            .set((
                users::email.eq(user.email().as_str()),
                users::hashed_password.eq(user.hashed_password().as_str()),
                users::display_name.eq(user.display_name()),
                users::is_active.eq(user.is_active()),
                users::updated_at.eq(user.updated_at()),
            ))
            .execute(&mut conn)
            .map_err(|e| AuthError::Internal(format!("Failed to update user: {}", e)))?;

        if updated_rows == 0 {
            return Err(AuthError::UserNotFound);
        }

        self.find_by_id(user.id().as_uuid())
    }
}

/// Convert database model to domain entity
fn db_user_to_domain(db_user: DbUser) -> User {
    User::from_persistence(
        db_user.id,
        db_user.email,
        db_user.hashed_password,
        db_user.display_name,
        db_user.is_active,
        db_user.created_at,
        db_user.updated_at,
    )
}
