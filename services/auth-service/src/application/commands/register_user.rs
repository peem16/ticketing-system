//! Register user use case

use crate::domain::auth::{PasswordHasher, UserRepository};
use crate::domain::error::AuthError;
use crate::domain::user::{Email, User};

/// Input for user registration
#[derive(Debug)]
pub struct RegisterUserCommand {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

/// Output after successful registration
#[derive(Debug)]
pub struct RegisterUserResult {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub display_name: Option<String>,
}

/// Use case for registering a new user
pub struct RegisterUserUseCase<'a, R: ?Sized, H: ?Sized> {
    user_repository: &'a R,
    password_hasher: &'a H,
}

impl<'a, R, H> RegisterUserUseCase<'a, R, H>
where
    R: UserRepository + ?Sized,
    H: PasswordHasher + ?Sized,
{
    /// Create a new use case instance
    pub fn new(user_repository: &'a R, password_hasher: &'a H) -> Self {
        Self {
            user_repository,
            password_hasher,
        }
    }

    /// Execute the registration
    ///
    /// # Errors
    /// - `AuthError::UserAlreadyExists` if email is taken
    /// - `AuthError::InvalidEmail` if email format is invalid
    /// - `AuthError::WeakPassword` if password doesn't meet requirements
    /// - `AuthError::Internal` on infrastructure failures
    pub fn execute(&self, command: RegisterUserCommand) -> Result<RegisterUserResult, AuthError> {
        // Validate email format
        let email = Email::new(&command.email)?;

        // Check if user already exists
        if self.user_repository.exists_by_email(email.as_str())? {
            return Err(AuthError::UserAlreadyExists);
        }

        // Validate password strength (basic check)
        if command.password.len() < 8 {
            return Err(AuthError::WeakPassword);
        }

        // Hash the password
        let hashed_password = self.password_hasher.hash(&command.password)?;

        // Create the user entity
        let user = User::new(email, hashed_password, command.display_name);

        // Persist the user
        let created_user = self.user_repository.create(&user)?;

        Ok(RegisterUserResult {
            user_id: created_user.id().as_uuid(),
            email: created_user.email().as_str().to_string(),
            display_name: created_user.display_name().map(String::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::HashedPassword;

    // Simple mock repository for testing
    struct MockUserRepository {
        existing_emails: Vec<String>,
    }

    impl UserRepository for MockUserRepository {
        fn find_by_id(&self, _id: uuid::Uuid) -> Result<User, AuthError> {
            Err(AuthError::UserNotFound)
        }

        fn find_by_email(&self, email: &str) -> Result<User, AuthError> {
            if self.existing_emails.contains(&email.to_string()) {
                Ok(User::from_persistence(
                    uuid::Uuid::new_v4(),
                    email.to_string(),
                    "hash".to_string(),
                    None,
                    true,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                ))
            } else {
                Err(AuthError::UserNotFound)
            }
        }

        fn exists_by_email(&self, email: &str) -> Result<bool, AuthError> {
            Ok(self.existing_emails.contains(&email.to_lowercase()))
        }

        fn create(&self, user: &User) -> Result<User, AuthError> {
            Ok(User::from_persistence(
                user.id().as_uuid(),
                user.email().as_str().to_string(),
                user.hashed_password().as_str().to_string(),
                user.display_name().map(String::from),
                user.is_active(),
                user.created_at(),
                user.updated_at(),
            ))
        }

        fn update(&self, _user: &User) -> Result<User, AuthError> {
            Err(AuthError::Internal("Not implemented".to_string()))
        }
    }

    // Simple mock hasher
    struct MockPasswordHasher;

    impl PasswordHasher for MockPasswordHasher {
        fn hash(&self, plain: &str) -> Result<HashedPassword, AuthError> {
            Ok(HashedPassword::from_hash(format!("hashed_{}", plain)))
        }

        fn verify(&self, plain: &str, hashed: &HashedPassword) -> Result<bool, AuthError> {
            Ok(hashed.as_str() == format!("hashed_{}", plain))
        }
    }

    #[test]
    fn test_register_success() {
        let repo = MockUserRepository {
            existing_emails: vec![],
        };
        let hasher = MockPasswordHasher;
        let use_case = RegisterUserUseCase::new(&repo, &hasher);

        let command = RegisterUserCommand {
            email: "new@example.com".to_string(),
            password: "ValidPass123".to_string(),
            display_name: Some("New User".to_string()),
        };

        let result = use_case.execute(command).unwrap();
        assert_eq!(result.email, "new@example.com");
        assert_eq!(result.display_name, Some("New User".to_string()));
    }

    #[test]
    fn test_register_email_exists() {
        let repo = MockUserRepository {
            existing_emails: vec!["existing@example.com".to_string()],
        };
        let hasher = MockPasswordHasher;
        let use_case = RegisterUserUseCase::new(&repo, &hasher);

        let command = RegisterUserCommand {
            email: "existing@example.com".to_string(),
            password: "ValidPass123".to_string(),
            display_name: None,
        };

        let result = use_case.execute(command);
        assert!(matches!(result, Err(AuthError::UserAlreadyExists)));
    }

    #[test]
    fn test_register_weak_password() {
        let repo = MockUserRepository {
            existing_emails: vec![],
        };
        let hasher = MockPasswordHasher;
        let use_case = RegisterUserUseCase::new(&repo, &hasher);

        let command = RegisterUserCommand {
            email: "test@example.com".to_string(),
            password: "short".to_string(),
            display_name: None,
        };

        let result = use_case.execute(command);
        assert!(matches!(result, Err(AuthError::WeakPassword)));
    }
}
