//! JWT Token 模块
//!
//! 提供 JWT Token 的生成和验证功能

use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use uuid::Uuid;

use crate::AuthError;

/// Token 过期时间：7 天
const TOKEN_EXPIRY_HOURS: i64 = 24 * 7;

/// 从环境变量获取 JWT 密钥，开发环境使用默认密钥
fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-only-secret-change-in-production".to_string())
        .into_bytes()
}

/// JWT Claims（声明）
///
/// 包含 Token 中存储的用户信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 ID
    pub sub: String,
    /// 用户邮箱
    pub email: String,
    /// 过期时间戳
    pub exp: i64,
    /// 签发时间戳
    pub iat: i64,
}

impl Claims {
    /// 创建新的 Claims
    ///
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `email` - 用户邮箱
    pub fn new(user_id: Uuid, email: &str) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: (now + Duration::hours(TOKEN_EXPIRY_HOURS)).timestamp(),
            iat: now.timestamp(),
        }
    }

    /// 获取用户 ID
    pub fn user_id(&self) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&self.sub).map_err(|_| AuthError::InvalidToken)
    }
}

/// JWT 服务
pub struct JwtService;

impl JwtService {
    /// 为用户生成 JWT Token
    ///
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `email` - 用户邮箱
    ///
    /// # 返回
    /// 生成的 JWT Token 字符串
    pub fn generate_token(user_id: Uuid, email: &str) -> Result<String, AuthError> {
        let claims = Claims::new(user_id, email);
        let secret = get_jwt_secret();

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&secret),
        ).map_err(|_| AuthError::InvalidToken)
    }

    /// 验证并解码 JWT Token
    ///
    /// # 参数
    /// * `token` - JWT Token 字符串
    ///
    /// # 返回
    /// 解码后的 Claims
    pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
        let secret = get_jwt_secret();

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(&secret),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AuthError::InvalidToken)
    }

    /// 从 Token 中提取用户 ID
    ///
    /// # 参数
    /// * `token` - JWT Token 字符串
    ///
    /// # 返回
    /// 用户 ID
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
