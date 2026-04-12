use axum::{routing::{post, get}, Router, Json, extract::State, Extension};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use rand::Rng;

use crate::state::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use models::User;
use auth::{hash_password, verify_password, ApiKeyGenerator, JwtService};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/admin-login", post(admin_login))
        .route("/logout", post(logout))
        .route("/send-sms", post(send_sms))
        .route("/verify-sms", post(verify_sms))
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
    pub is_admin: bool,
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
            is_admin: user.is_admin,
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
        .ok_or(ApiError::InvalidCredentials)?;

    // Verify password
    let password_hash = user.password_hash.as_ref()
        .ok_or(ApiError::InvalidCredentials)?;

    let is_valid = verify_password(&request.password, password_hash)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Password verification failed")))?;

    if !is_valid {
        return Err(ApiError::InvalidCredentials);
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
            is_admin: user.is_admin,
        },
        token,
    }))
}

/// POST /v1/auth/admin-login
/// Admin-only login endpoint. Returns Forbidden if the user is not an admin.
pub async fn admin_login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Get user
    let user = state.db.get_user_by_email(&request.email)
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Database error")))?
        .ok_or(ApiError::InvalidCredentials)?;

    // Verify password
    let password_hash = user.password_hash.as_ref()
        .ok_or(ApiError::InvalidCredentials)?;

    let is_valid = verify_password(&request.password, password_hash)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Password verification failed")))?;

    if !is_valid {
        return Err(ApiError::InvalidCredentials);
    }

    // Check admin privilege
    if !user.is_admin {
        return Err(ApiError::Forbidden);
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
            is_admin: user.is_admin,
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

// ============ SMS Authentication ============

/// Send SMS request body
#[derive(Debug, Deserialize)]
pub struct SendSmsRequest {
    pub phone: String,
}

/// Send SMS response
#[derive(Debug, Serialize)]
pub struct SendSmsResponse {
    pub message: String,
    pub seconds_valid: i64,
}

/// POST /v1/auth/send-sms
/// Send verification code to phone number using阿里云短信服务
pub async fn send_sms(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendSmsRequest>,
) -> Result<Json<SendSmsResponse>, ApiError> {
    let phone = request.phone.trim();

    // Validate phone number (basic validation)
    if phone.len() < 10 || phone.len() > 15 {
        return Err(ApiError::InvalidRequest("Invalid phone number format".to_string()));
    }

    // Check rate limit
    let (allowed, wait_seconds) = state.redis.check_sms_rate_limit(phone)
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to check rate limit")))?;

    if !allowed {
        return Err(ApiError::SmsRateLimitExceeded);
    }

    // Generate 6-digit code
    let code: u32 = rand::thread_rng().gen_range(100000..999999);

    // Store code in Redis
    state.redis.store_sms_code(phone, &code.to_string())
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to store SMS code")))?;

    // Send SMS via阿里云短信服务
    let sms_result = send_alibaba_sms(phone, &code.to_string()).await;

    if sms_result.is_err() {
        tracing::error!("Failed to send SMS to {}: {:?}", phone, sms_result.err());
        // Don't expose the error to the client - just log it
        // In production, you might want to return a different error
        return Err(ApiError::SmsSendFailed);
    }

    Ok(Json(SendSmsResponse {
        message: "Verification code sent successfully".to_string(),
        seconds_valid: 300, // 5 minutes
    }))
}

/// Verify SMS request body
#[derive(Debug, Deserialize)]
pub struct VerifySmsRequest {
    pub phone: String,
    pub code: String,
}

/// POST /v1/auth/verify-sms
/// Verify SMS code and return auth token (auto-register if needed)
pub async fn verify_sms(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VerifySmsRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let phone = request.phone.trim();
    let code = request.code.trim();

    // Validate inputs
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::InvalidSmsCode);
    }

    // Get stored code
    let stored_code = state.redis.get_sms_code(phone)
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to retrieve SMS code")))?;

    // Verify code matches
    match stored_code {
        Some(stored) if stored == code => {
            // Delete the code after successful verification
            let _ = state.redis.delete_sms_code(phone).await;
        }
        _ => {
            return Err(ApiError::InvalidSmsCode);
        }
    }

    // Check if user exists with this phone
    let existing_user = state.db.get_user_by_phone(phone)
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Database error")))?;

    let user = if let Some(mut user) = existing_user {
        // User exists - generate token
        user
    } else {
        // Auto-register new user with phone
        let mut new_user = User::new(format!("{}@nexus.sms", phone));
        new_user = new_user.with_phone(phone.to_string());

        state.db.create_user(&new_user)
            .await
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to create user")))?;

        new_user
    };

    // Generate token using phone as identifier
    let token = JwtService::generate_token(user.id, phone)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to generate token")))?;

    Ok(Json(AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email.clone(),
            phone: user.phone,
            subscription_plan: user.subscription_plan.as_str().to_string(),
            is_admin: user.is_admin,
        },
        token,
    }))
}

/// Send SMS via Alibaba Cloud (阿里云短信服务)
async fn send_alibaba_sms(phone: &str, code: &str) -> Result<(), ApiError> {
    // Get阿里云 SMS configuration from environment
    let access_key_id = std::env::var("ALIYUN_ACCESS_KEY_ID")
        .map_err(|_| ApiError::SmsSendFailed)?;
    let access_key_secret = std::env::var("ALIYUN_ACCESS_KEY_SECRET")
        .map_err(|_| ApiError::SmsSendFailed)?;
    let sign_name = std::env::var("ALIYUN_SMS_SIGN_NAME")
        .map_err(|_| ApiError::SmsSendFailed)?;
    let template_code = std::env::var("ALIYUN_SMS_TEMPLATE_CODE")
        .map_err(|_| ApiError::SmsSendFailed)?;

    // Construct the request
    let params = serde_json::json!({
        "PhoneNumbers": phone,
        "SignName": sign_name,
        "TemplateCode": template_code,
        "TemplateParam": serde_json::json!({
            "code": code
        })
    });

    // Build the signature (simplified - in production use proper阿里云 SDK)
    // This is a placeholder implementation
    let client = reqwest::Client::new();

    let response = client
        .post("https://dysmsapi.aliyuncs.com/")
        .json(&params)
        .send()
        .await
        .map_err(|_| ApiError::SmsSendFailed)?;

    if response.status().is_success() {
        Ok(())
    } else {
        tracing::error!("阿里云 SMS API error: {:?}", response.text().await);
        Err(ApiError::SmsSendFailed)
    }
}
