//! 认证路由模块
//! 处理用户注册、登录、登出和短信认证相关请求
//!
//! 路由列表：
//! - POST /register - 用户注册
//! - POST /login - 用户登录
//! - POST /admin-login - 管理员登录
//! - POST /logout - 用户登出
//! - POST /send-sms - 发送短信验证码
//! - POST /verify-sms - 验证短信验证码

use axum::{routing::{post, get}, Router, Json, extract::State, Extension};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use rand::Rng;

use crate::state::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use models::User;
use auth::{hash_password, verify_password, ApiKeyGenerator, JwtService};

/// 创建认证路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/admin-login", post(admin_login))
        .route("/logout", post(logout))
        .route("/send-sms", post(send_sms))
        .route("/verify-sms", post(verify_sms))
}

/// 注册请求体
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// 邮箱地址
    pub email: String,
    /// 密码
    pub password: String,
    /// 手机号（可选）
    pub phone: Option<String>,
}

/// 认证响应体
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// 用户信息
    pub user: UserResponse,
    /// JWT Token
    pub token: String,
}

/// 用户信息响应
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// 用户 ID
    pub id: String,
    /// 邮箱
    pub email: String,
    /// 手机号
    pub phone: Option<String>,
    /// 订阅套餐
    pub subscription_plan: String,
    /// 是否为管理员
    pub is_admin: bool,
}

/// POST /v1/auth/register
///
/// 用户注册接口
///
/// # 参数
/// * `Json(request)` - 注册请求，包含 email、password 和可选的 phone
///
/// # 返回
/// 成功返回 AuthResponse，包含用户信息和 JWT Token
///
/// # 错误处理
/// - 邮箱已存在返回 UserAlreadyExists
/// - 密码哈希失败返回 InternalError
/// - 创建用户失败返回 InternalError
/// - 生成 Token 失败返回 InternalError
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

/// 登录请求体
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// 邮箱地址
    pub email: String,
    /// 密码
    pub password: String,
}

/// POST /v1/auth/login
///
/// 用户登录接口
///
/// # 参数
/// * `Json(request)` - 登录请求，包含 email 和 password
///
/// # 返回
/// 成功返回 AuthResponse，包含用户信息和 JWT Token
///
/// # 错误处理
/// - 用户不存在返回 InvalidCredentials
/// - 密码错误返回 InvalidCredentials
/// - 生成 Token 失败返回 InternalError
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
///
/// 管理员登录接口
///
/// # 说明
/// 只有管理员用户才能登录，非管理员用户会返回 Forbidden 错误
///
/// # 参数
/// * `Json(request)` - 登录请求，包含 email 和 password
///
/// # 返回
/// 成功返回 AuthResponse，包含用户信息和 JWT Token
///
/// # 错误处理
/// - 用户不存在返回 InvalidCredentials
/// - 密码错误返回 InvalidCredentials
/// - 非管理员用户返回 Forbidden
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
///
/// 用户登出接口
///
/// # 说明
/// 实际实现中应该在 Redis 中使 Token 失效
pub async fn logout(
    State(_state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // In a real implementation, we would invalidate the token in Redis
    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

// ============ SMS 短信认证 ============

/// 发送短信请求体
#[derive(Debug, Deserialize)]
pub struct SendSmsRequest {
    /// 手机号
    pub phone: String,
}

/// 发送短信响应
#[derive(Debug, Serialize)]
pub struct SendSmsResponse {
    /// 响应消息
    pub message: String,
    /// 验证码有效时间（秒）
    pub seconds_valid: i64,
}

/// POST /v1/auth/send-sms
///
/// 发送短信验证码到手机号
///
/// # 说明
/// 使用阿里云短信服务发送验证码
///
/// # 参数
/// * `Json(request)` - 请求体，包含手机号
///
/// # 限制
/// - 手机号格式校验（10-15位）
/// - 短信发送频率限制（通过 Redis 检查）
///
/// # 错误处理
/// - 手机号格式无效返回 InvalidRequest
/// - 发送频率超限返回 SmsRateLimitExceeded
/// - 短信发送失败返回 SmsSendFailed
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

/// 验证短信请求体
#[derive(Debug, Deserialize)]
pub struct VerifySmsRequest {
    /// 手机号
    pub phone: String,
    /// 验证码（6位数字）
    pub code: String,
}

/// POST /v1/auth/verify-sms
///
/// 验证短信验证码并返回认证 Token
///
/// # 说明
/// 如果该手机号未注册，会自动创建新用户（手机号作为唯一标识）
///
/// # 参数
/// * `Json(request)` - 请求体，包含手机号和验证码
///
/// # 验证流程
/// 1. 验证验证码格式（6位数字）
/// 2. 从 Redis 获取存储的验证码并比对
/// 3. 验证成功后删除验证码（防止重复使用）
/// 4. 检查用户是否已注册
/// 5. 未注册则自动创建用户
/// 6. 生成 JWT Token 返回
///
/// # 错误处理
/// - 验证码格式错误返回 InvalidSmsCode
/// - 验证码不匹配或已过期返回 InvalidSmsCode
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

/// 通过阿里云短信服务发送短信
    ///
    /// # 环境变量
    /// - ALIYUN_ACCESS_KEY_ID: 阿里云 Access Key ID
    /// - ALIYUN_ACCESS_KEY_SECRET: 阿里云 Access Key Secret
    /// - ALIYUN_SMS_SIGN_NAME: 短信签名
    /// - ALIYUN_SMS_TEMPLATE_CODE: 短信模板 CODE
    ///
    /// # 参数
    /// * `phone` - 目标手机号
    /// * `code` - 验证码
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
