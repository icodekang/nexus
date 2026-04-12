use axum::{http::StatusCode, Json, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

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
