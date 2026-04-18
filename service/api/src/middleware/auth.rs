use axum::{
    extract::Request,
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

/// 验证 API Key 中间件
///
/// 从 Authorization header 中提取并验证 API Key
///
/// # 请求头格式
/// - `Authorization: Bearer <api_key>`
///
/// # 错误处理
/// - 缺少 Authorization header 返回 Unauthorized
/// - 不是 Bearer 格式返回 InvalidApiKey
/// - API Key 格式无效返回 InvalidApiKey
/// - API Key 不存在或过期返回相应错误
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

/// 从请求中提取 Token
    ///
    /// 支持两种格式：
    /// - `Authorization: Bearer <token>`  (OpenAI SDK)
    /// - `x-api-key: <token>`             (Anthropic SDK, LiteLLM, curl 等)
    ///
    /// # 参数
    /// * `req` - HTTP 请求引用
    ///
    /// # 返回
    /// 如果找到有效的 token 返回 Some(String)，否则返回 None
fn extract_token(req: &Request) -> Option<String> {
    // Authorization: Bearer <token>
    let bearer = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .filter(|h| h.starts_with("Bearer "))
        .map(|h| h[7..].to_string());

    if bearer.is_some() {
        return bearer;
    }

    // x-api-key: <token>  (Anthropic SDK)
    req.headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// JWT 或 API Key 验证中间件
    ///
    /// 验证 Authorization header 中的 token，支持两种格式：
    /// 1. API Key 格式（优先）：适用于 API Key 用户
    /// 2. JWT 格式（回退）：适用于 JWT Token 用户
    ///
    /// 支持的 header 格式：
    /// - `Authorization: Bearer <key>` - API Key 或 JWT
    /// - `x-api-key: <key>` - API Key（Anthropic SDK 兼容）
    ///
    /// # 验证流程
    /// 1. 提取 token
    /// 2. 如果是 API Key 格式，先尝试 API Key 验证
    /// 3. API Key 验证失败则回退到 JWT 验证
    /// 4. 验证成功后在请求扩展中插入 AuthContext
    ///
    /// # 错误处理
    /// - 缺少 token 返回 Unauthorized
    /// - API Key 过期返回 SubscriptionExpired
    /// - JWT 无效返回 Unauthorized
    /// - 用户不存在返回 Unauthorized
pub async fn validate_jwt_or_api_key(
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_token(&req).ok_or(ApiError::Unauthorized)?;

    // Get app state
    let state = req.extensions().get::<Arc<AppState>>()
        .cloned()
        .ok_or(ApiError::Internal(anyhow::anyhow!("Missing app state")))?;

    // Try API key path first
    if ApiKeyGenerator::validate_format(&token) {
        let validator = ApiKeyValidator::new(&state.db);
        match validator.validate(&token).await {
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
    let claims = JwtService::validate_token(&token)
        .map_err(|_| ApiError::Unauthorized)?;

    // Check if token is blacklisted
    if state.redis.is_token_blacklisted(&token).await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to check token blacklist")))?
    {
        return Err(ApiError::Unauthorized);
    }

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

/// 管理员权限验证中间件
    ///
    /// 要求已通过 `validate_jwt_or_api_key` 认证的用户具有管理员权限
    /// 如果用户不是管理员，返回 Forbidden 错误
    ///
    /// # 使用方式
    /// 此中间件必须在 `validate_jwt_or_api_key` 之后使用
    ///
    /// # 错误处理
    /// - 没有 AuthContext 返回 Unauthorized
    /// - 用户不是管理员返回 Forbidden
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
