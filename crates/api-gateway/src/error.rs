use axum::{http::StatusCode, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Insufficient credits: required {required}, available {available}")]
    InsufficientCredits { required: String, available: String },

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

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
            ApiError::InsufficientCredits { .. } => (
                StatusCode::PAYMENT_REQUIRED,
                "insufficient_credits",
                self.to_string(),
            ),
            ApiError::InvalidApiKey => (
                StatusCode::UNAUTHORIZED,
                "invalid_api_key",
                "Invalid API key",
            ),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Unauthorized",
            ),
            ApiError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limit_exceeded",
                "Rate limit exceeded",
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
            ApiError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                e.to_string(),
            ),
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
