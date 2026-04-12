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
use auth::{ApiKeyGenerator, ApiKeyValidator, JwtService};

/// Authentication context extracted from API key or JWT
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

/// Validate either an API key or a JWT token from the Authorization header.
/// Tries API key format first; if not an API key, falls back to JWT validation.
pub async fn validate_jwt_or_api_key(
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::Unauthorized);
    }

    let token = &auth_header[7..];

    // Get app state
    let state = req.extensions().get::<Arc<AppState>>()
        .cloned()
        .ok_or(ApiError::Internal(anyhow::anyhow!("Missing app state")))?;

    // Try API key path first
    if ApiKeyGenerator::validate_format(token) {
        let validator = ApiKeyValidator::new(&state.db);
        match validator.validate(token).await {
            Ok((api_key, user)) => {
                req.extensions_mut().insert(AuthContext {
                    user,
                    api_key_id: Some(api_key.id),
                });
                return Ok(next.run(req).await);
            }
            Err(auth::AuthError::SubscriptionExpired) => {
                // For API keys, subscription must be active
                return Err(ApiError::SubscriptionExpired);
            }
            Err(_) => {
                // API key validation failed — fall through to JWT
            }
        }
    }

    // JWT path
    let claims = JwtService::validate_token(token)
        .map_err(|_| ApiError::Unauthorized)?;

    let user_id = claims.user_id()
        .map_err(|_| ApiError::Unauthorized)?;

    let user = state.db.get_user_by_id(user_id).await
        .map_err(|e| {
            tracing::error!("Database error during JWT auth: {:?}", e);
            ApiError::Internal(anyhow::anyhow!("Database error"))
        })?
        .ok_or(ApiError::Unauthorized)?;

    req.extensions_mut().insert(AuthContext {
        user,
        api_key_id: None,
    });

    Ok(next.run(req).await)
}

/// Middleware that requires the authenticated user to be an admin.
/// Must be applied after `validate_jwt_or_api_key`.
pub async fn require_admin(
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth = req.extensions().get::<AuthContext>()
        .cloned()
        .ok_or(ApiError::Unauthorized)?;

    if !auth.user.is_admin {
        return Err(ApiError::Forbidden);
    }

    Ok(next.run(req).await)
}
