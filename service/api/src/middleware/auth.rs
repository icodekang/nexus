use axum::{
    extract::{Request, Extension},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::error::ApiError;
use crate::state::AppState;
use crate::middleware::AuthContext;
use auth::{ApiKeyGenerator, BearerValidator};

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

    // Validate the API key
    let validator = BearerValidator::new(&state.db);
    let user = validator.validate(auth_header).await?;

    // Insert auth context into request
    req.extensions_mut().insert(AuthContext {
        user,
        api_key_id: None,
    });

    Ok(next.run(req).await)
}
