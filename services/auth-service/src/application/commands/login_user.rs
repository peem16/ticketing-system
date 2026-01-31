//! Login user use case

use crate::domain::auth::{PasswordHasher, TokenService, UserRepository};
use crate::domain::error::AuthError;

/// Input for user login
#[derive(Debug)]
pub struct LoginUserCommand {
    pub email: String,
    pub password: String,
}

/// Output after successful login
#[derive(Debug)]
pub struct LoginUserResult {
    pub token: String,
    pub user_id: uuid::Uuid,
    pub email: String,
    pub display_name: Option<String>,
}

/// Use case for user login
pub struct LoginUserUseCase<'a, R: ?Sized, H: ?Sized, T: ?Sized> {
    user_repository: &'a R,
    password_hasher: &'a H,
    token_service: &'a T,
}

impl<'a, R, H, T> LoginUserUseCase<'a, R, H, T>
where
    R: UserRepository + ?Sized,
    H: PasswordHasher + ?Sized,
    T: TokenService + ?Sized,
{
    /// Create a new use case instance
    pub fn new(user_repository: &'a R, password_hasher: &'a H, token_service: &'a T) -> Self {
        Self {
            user_repository,
            password_hasher,
            token_service,
        }
    }

    /// Execute the login
    ///
    /// # Errors
    /// - `AuthError::InvalidCredentials` if email/password is wrong
    /// - `AuthError::AccountInactive` if user account is deactivated
    /// - `AuthError::Internal` on infrastructure failures
    pub fn execute(&self, command: LoginUserCommand) -> Result<LoginUserResult, AuthError> {
        // Find user by email
        let user = self
            .user_repository
            .find_by_email(&command.email)
            .map_err(|e| match e {
                AuthError::UserNotFound => AuthError::InvalidCredentials,
                other => other,
            })?;

        // Check if account is active
        if !user.is_active() {
            return Err(AuthError::AccountInactive);
        }

        // Verify password
        let is_valid = self
            .password_hasher
            .verify(&command.password, user.hashed_password())?;

        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate JWT token
        let token = self.token_service.create_token(&user)?;

        Ok(LoginUserResult {
            token,
            user_id: user.id().as_uuid(),
            email: user.email().as_str().to_string(),
            display_name: user.display_name().map(String::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth::TokenData;
    use crate::domain::user::{HashedPassword, User};

    // Mock repository
    struct MockUserRepository {
        user: Option<User>,
    }

    impl UserRepository for MockUserRepository {
        fn find_by_id(&self, _id: uuid::Uuid) -> Result<User, AuthError> {
            self.user.clone().ok_or(AuthError::UserNotFound)
        }

        fn find_by_email(&self, email: &str) -> Result<User, AuthError> {
            match &self.user {
                Some(u) if u.email().as_str() == email => Ok(u.clone()),
                _ => Err(AuthError::UserNotFound),
            }
        }

        fn exists_by_email(&self, email: &str) -> Result<bool, AuthError> {
            Ok(self.user.as_ref().map_or(false, |u| u.email().as_str() == email))
        }

        fn create(&self, _user: &User) -> Result<User, AuthError> {
            Err(AuthError::Internal("Not implemented".to_string()))
        }

        fn update(&self, _user: &User) -> Result<User, AuthError> {
            Err(AuthError::Internal("Not implemented".to_string()))
        }
    }

    // Mock password hasher that expects "correct_password"
    struct MockPasswordHasher;

    impl PasswordHasher for MockPasswordHasher {
        fn hash(&self, plain: &str) -> Result<HashedPassword, AuthError> {
            Ok(HashedPassword::from_hash(format!("hashed_{}", plain)))
        }

        fn verify(&self, plain: &str, _hashed: &HashedPassword) -> Result<bool, AuthError> {
            Ok(plain == "correct_password")
        }
    }

    // Mock token service
    struct MockTokenService;

    impl TokenService for MockTokenService {
        fn create_token(&self, _user: &User) -> Result<String, AuthError> {
            Ok("mock_token".to_string())
        }

        fn validate_token(&self, _token: &str) -> Result<TokenData, AuthError> {
            Err(AuthError::InvalidToken)
        }
    }

    fn create_test_user() -> User {
        use crate::domain::user::Email;
        let email = Email::new("test@example.com").unwrap();
        let password = HashedPassword::from_hash("hashed_correct_password".to_string());
        User::new(email, password, Some("Test".to_string()))
    }

    #[test]
    fn test_login_success() {
        let repo = MockUserRepository {
            user: Some(create_test_user()),
        };
        let hasher = MockPasswordHasher;
        let token_service = MockTokenService;
        let use_case = LoginUserUseCase::new(&repo, &hasher, &token_service);

        let command = LoginUserCommand {
            email: "test@example.com".to_string(),
            password: "correct_password".to_string(),
        };

        let result = use_case.execute(command).unwrap();
        assert_eq!(result.token, "mock_token");
        assert_eq!(result.email, "test@example.com");
    }

    #[test]
    fn test_login_wrong_password() {
        let repo = MockUserRepository {
            user: Some(create_test_user()),
        };
        let hasher = MockPasswordHasher;
        let token_service = MockTokenService;
        let use_case = LoginUserUseCase::new(&repo, &hasher, &token_service);

        let command = LoginUserCommand {
            email: "test@example.com".to_string(),
            password: "wrong_password".to_string(),
        };

        let result = use_case.execute(command);
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[test]
    fn test_login_user_not_found() {
        let repo = MockUserRepository { user: None };
        let hasher = MockPasswordHasher;
        let token_service = MockTokenService;
        let use_case = LoginUserUseCase::new(&repo, &hasher, &token_service);

        let command = LoginUserCommand {
            email: "nonexistent@example.com".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(command);
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }
}
