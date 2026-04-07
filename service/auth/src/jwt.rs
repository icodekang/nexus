use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use uuid::Uuid;

use crate::AuthError;

const JWT_SECRET: &[u8] = b"your-256-bit-secret-change-in-production";
const TOKEN_EXPIRY_HOURS: i64 = 24 * 7; // 7 days

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user_id
    pub email: String,
    pub exp: i64,           // expiration time
    pub iat: i64,           // issued at
}

impl Claims {
    pub fn new(user_id: Uuid, email: &str) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: (now + Duration::hours(TOKEN_EXPIRY_HOURS)).timestamp(),
            iat: now.timestamp(),
        }
    }

    pub fn user_id(&self) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&self.sub).map_err(|_| AuthError::InvalidToken)
    }
}

/// JWT Service
pub struct JwtService;

impl JwtService {
    /// Generate a JWT token for a user
    pub fn generate_token(user_id: Uuid, email: &str) -> Result<String, AuthError> {
        let claims = Claims::new(user_id, email);
        
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        ).map_err(|_| AuthError::InvalidToken)
    }

    /// Validate and decode a JWT token
    pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AuthError::InvalidToken)
    }

    /// Extract user_id from a token
    pub fn get_user_id(token: &str) -> Result<Uuid, AuthError> {
        let claims = Self::validate_token(token)?;
        claims.user_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate() {
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        
        let token = JwtService::generate_token(user_id, email).unwrap();
        let claims = JwtService::validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_invalid_token() {
        assert!(JwtService::validate_token("invalid.token.here").is_err());
    }
}
