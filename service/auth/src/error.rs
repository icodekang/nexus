use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("API key not found")]
    ApiKeyNotFound,
    
    #[error("API key invalid")]
    ApiKeyInvalid,
    
    #[error("Subscription expired")]
    SubscriptionExpired,
    
    #[error("Password hash error")]
    PasswordHashError,
}

impl From<bcrypt::BcryptError> for AuthError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AuthError::PasswordHashError
    }
}
