use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};

const JWT_SECRET: &[u8] = b"your-256-bit-secret-key-change-in-production";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user_id
    pub email: String,
    pub exp: i64,           // expiration time
}

pub struct JwtService;

impl JwtService {
    pub fn new() -> Self {
        Self
    }

    /// Generate a JWT token for a user
    pub fn generate_token(&self, user_id: &str, email: &str) -> Result<String, JwtError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        ).map_err(|_| JwtError::TokenGenerationFailed)
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| JwtError::InvalidToken)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Token generation failed")]
    TokenGenerationFailed,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    ExpiredToken,
}
