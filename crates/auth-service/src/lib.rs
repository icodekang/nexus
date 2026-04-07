pub mod keygen;
pub mod validator;
pub mod jwt;

use models::user::User;

pub struct AuthService;

impl AuthService {
    pub fn new() -> Self {
        Self
    }

    /// Validate user credentials
    pub fn authenticate(&self, email: &str, password: &str) -> Result<User, AuthError> {
        // TODO: Check against database
        if email.is_empty() || password.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }
        Ok(User::new(email.to_string()))
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,
}
