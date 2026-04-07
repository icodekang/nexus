use axum::{
    extract::{Request, Extension},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

use crate::error::ApiError;
use crate::routes::v1::chat::AuthContext;

pub async fn validate_api_key(
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    if !auth_header.starts_with("Bearer sk-nova-") {
        return Err(ApiError::InvalidApiKey);
    }

    // TODO: Validate against database
    let auth = AuthContext {
        user_id: "user_123".to_string(),
        credits: rust_decimal::Decimal::new(10000, 4), // $10.00
    };

    req.extensions_mut().insert(auth);

    Ok(next.run(req).await)
}
