//! API 错误类型定义
//! 所有 API 错误都定义在此模块中
//!
//! 错误分类：
//! - 认证错误：InvalidApiKey、InvalidCredentials、Unauthorized
//! - 业务错误：SubscriptionExpired、RateLimitExceeded、ModelNotFound
//! - 第三方错误：ProviderError、SmsSendFailed
//! - 客户端错误：InvalidRequest、NotImplemented

use axum::{http::StatusCode, Json, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

/// API 错误枚举
///
/// # 错误类型
/// - `ModelNotFound` - 模型未找到
/// - `InvalidApiKey` - API Key 无效
/// - `InvalidCredentials` - 邮箱或密码错误
/// - `Unauthorized` - 未授权访问
/// - `Forbidden` - 禁止访问（权限不足）
/// - `SubscriptionExpired` - 订阅已过期
/// - `RateLimitExceeded` - 请求频率超限
/// - `ProviderError` - 上游 provider 错误
/// - `NotImplemented` - 功能未实现
/// - `InvalidRequest` - 请求参数无效
/// - `UserAlreadyExists` - 用户已存在
/// - `SmsSendFailed` - 短信发送失败
/// - `SmsRateLimitExceeded` - 短信发送频率超限
/// - `InvalidSmsCode` - 短信验证码无效
/// - `PhoneAlreadyRegistered` - 手机号已注册
/// - `UserNotFound` - 用户未找到
/// - `Internal` - 内部错误
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Invalid email or password")]
    InvalidCredentials,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Subscription expired")]
    SubscriptionExpired,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("SMS send failed")]
    SmsSendFailed,

    #[error("SMS rate limit exceeded")]
    SmsRateLimitExceeded,

    #[error("Invalid SMS code")]
    InvalidSmsCode,

    #[error("Phone number already registered")]
    PhoneAlreadyRegistered,

    #[error("User not found")]
    UserNotFound,

    #[error("Login failed: {0}")]
    LoginFailed(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Page structure changed, manual intervention required")]
    PageStructureChanged,

    #[error("Cloudflare challenge detected, manual intervention required")]
    CloudflareChallenge,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_code, message) = match &self {
            ApiError::ModelNotFound(m) => (
                StatusCode::NOT_FOUND,
                "model_not_found",
                format!("Model '{}' not found", m),
            ),
            ApiError::InvalidApiKey => (
                StatusCode::UNAUTHORIZED,
                "invalid_api_key",
                "Invalid API key".to_string(),
            ),
            ApiError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "Invalid email or password".to_string(),
            ),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Unauthorized".to_string(),
            ),
            ApiError::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "Forbidden".to_string(),
            ),
            ApiError::SubscriptionExpired => (
                StatusCode::FORBIDDEN,
                "subscription_expired",
                "Subscription expired. Please renew your subscription.".to_string(),
            ),
            ApiError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limit_exceeded",
                "Rate limit exceeded. Please try again later.".to_string(),
            ),
            ApiError::ProviderError(e) => (
                StatusCode::BAD_GATEWAY,
                "provider_error",
                e.clone(),
            ),
            ApiError::NotImplemented(e) => (
                StatusCode::NOT_IMPLEMENTED,
                "not_implemented",
                e.clone(),
            ),
            ApiError::InvalidRequest(e) => (
                StatusCode::BAD_REQUEST,
                "invalid_request",
                e.clone(),
            ),
            ApiError::UserAlreadyExists => (
                StatusCode::CONFLICT,
                "user_already_exists",
                "User with this email already exists".to_string(),
            ),
            ApiError::SmsSendFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "sms_send_failed",
                "Failed to send SMS. Please try again later.".to_string(),
            ),
            ApiError::SmsRateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "sms_rate_limit_exceeded",
                "Too many SMS requests. Please try again later.".to_string(),
            ),
            ApiError::InvalidSmsCode => (
                StatusCode::UNAUTHORIZED,
                "invalid_sms_code",
                "Invalid or expired verification code.".to_string(),
            ),
            ApiError::PhoneAlreadyRegistered => (
                StatusCode::CONFLICT,
                "phone_already_registered",
                "This phone number is already registered.".to_string(),
            ),
            ApiError::UserNotFound => (
                StatusCode::NOT_FOUND,
                "user_not_found",
                "User not found.".to_string(),
            ),
            ApiError::LoginFailed(msg) => (
                StatusCode::UNAUTHORIZED,
                "login_failed",
                msg.clone(),
            ),
            ApiError::SessionExpired => (
                StatusCode::UNAUTHORIZED,
                "session_expired",
                "Session expired. Please login again.".to_string(),
            ),
            ApiError::PageStructureChanged => (
                StatusCode::BAD_GATEWAY,
                "page_structure_changed",
                "Login page structure changed. Please update selectors.".to_string(),
            ),
            ApiError::CloudflareChallenge => (
                StatusCode::BAD_GATEWAY,
                "cloudflare_challenge",
                "Cloudflare challenge detected. Manual browser login required.".to_string(),
            ),
            ApiError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred. Please try again later.".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": {
                "message": message,
                "type": error_code,
                "code": error_code
            }
        }));

        (status, body).into_response()
    }
}
