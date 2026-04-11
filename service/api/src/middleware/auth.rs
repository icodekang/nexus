use axum::{
    extract::{Request, Extension},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;
use models::User;
use auth::{ApiKeyGenerator, ApiKeyValidator, BearerValidator};

/// Authentication context extracted from API key
#[derive(Clone)]
pub struct AuthContext {
    pub user: User,
    pub api_key_id: Option<Uuid>,
}

/// Extract and validate API key from Authorization header
pub async fn validate_api_key(
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::InvalidApiKey);
    }

    let key = &auth_header[7..];

    // Validate key format
    if !ApiKeyGenerator::validate_format(key) {
        return Err(ApiError::InvalidApiKey);
    }

    // Get app state
    let state = req.extensions().get::<Arc<AppState>>()
        .cloned()
        .ok_or(ApiError::Internal(anyhow::anyhow!("Missing app state")))?;

    // Validate the API key (pass plain key, not full auth header)
    let validator = ApiKeyValidator::new(&state.db);
    let (api_key, user) = validator.validate(key).await
        .map_err(|e| match e {
            auth::AuthError::ApiKeyNotFound => ApiError::InvalidApiKey,
            auth::AuthError::ApiKeyInvalid => ApiError::InvalidApiKey,
            auth::AuthError::UserNotFound => ApiError::Unauthorized,
            auth::AuthError::SubscriptionExpired => ApiError::SubscriptionExpired,
            auth::AuthError::InvalidToken => ApiError::Unauthorized,
            _ => ApiError::Unauthorized,
        })?;

    // Insert auth context into request
    req.extensions_mut().insert(AuthContext {
        user,
        api_key_id: Some(api_key.id),
    });

    Ok(next.run(req).await)
}
