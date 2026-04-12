use axum::{routing::{post, get}, Router, Json, extract::State, Extension};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use models::User;
use auth::{hash_password, verify_password, ApiKeyGenerator, JwtService};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
}

/// Register request body
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
}

/// Register response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub phone: Option<String>,
    pub subscription_plan: String,
}

/// POST /v1/auth/register
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Check if user already exists
    if let Ok(Some(_)) = state.db.get_user_by_email(&request.email).await {
        return Err(ApiError::UserAlreadyExists);
    }

    // Hash password
    let password_hash = hash_password(&request.password)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to hash password")))?;

    // Create user
    let mut user = User::new(request.email.clone());
    user = user.with_password(password_hash);
    if let Some(phone) = request.phone {
        user = user.with_phone(phone);
    }

    state.db.create_user(&user).await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to create user")))?;

    // Generate token
    let token = JwtService::generate_token(user.id, &user.email)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to generate token")))?;

    Ok(Json(AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            phone: user.phone,
            subscription_plan: user.subscription_plan.as_str().to_string(),
        },
        token,
    }))
}

/// Login request body
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// POST /v1/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Get user
    let user = state.db.get_user_by_email(&request.email)
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Database error")))?
        .ok_or(ApiError::InvalidApiKey)?;

    // Verify password
    let password_hash = user.password_hash.as_ref()
        .ok_or(ApiError::InvalidApiKey)?;
    
    let is_valid = verify_password(&request.password, password_hash)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Password verification failed")))?;
    
    if !is_valid {
        return Err(ApiError::InvalidApiKey);
    }

    // Generate token
    let token = JwtService::generate_token(user.id, &user.email)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to generate token")))?;

    Ok(Json(AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            phone: user.phone,
            subscription_plan: user.subscription_plan.as_str().to_string(),
        },
        token,
    }))
}

/// POST /v1/auth/logout
pub async fn logout(
    State(_state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // In a real implementation, we would invalidate the token in Redis
    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
