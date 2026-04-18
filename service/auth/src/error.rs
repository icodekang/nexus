//! 认证错误模块

use thiserror::Error;

/// 认证错误枚举
///
/// 包含认证过程中可能出现的各种错误情况
#[derive(Error, Debug)]
pub enum AuthError {
    /// 凭据无效（用户名或密码错误）
    #[error("无效的凭据")]
    InvalidCredentials,

    /// 用户不存在
    #[error("用户不存在")]
    UserNotFound,

    /// 用户已存在
    #[error("用户已存在")]
    UserAlreadyExists,

    /// Token 无效
    #[error("无效的 Token")]
    InvalidToken,

    /// Token 已过期
    #[error("Token 已过期")]
    TokenExpired,

    /// API Key 不存在
    #[error("API Key 不存在")]
    ApiKeyNotFound,

    /// API Key 无效
    #[error("API Key 无效")]
    ApiKeyInvalid,

    /// 订阅已过期
    #[error("订阅已过期")]
    SubscriptionExpired,

    /// 密码哈希错误
    #[error("密码哈希错误")]
    PasswordHashError,
}

impl From<bcrypt::BcryptError> for AuthError {
    fn from(_err: bcrypt::BcryptError) -> Self {
        AuthError::PasswordHashError
    }
}
