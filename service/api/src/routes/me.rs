#![allow(unused_imports)]
use axum::{routing::{get, post, delete}, Router, Json, Extension, extract::State};
use std::sync::Arc;
use serde::Deserialize;

use crate::state::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use models::{ApiKey, SubscriptionPlan, subscription::SubscriptionPlanInfo};
use auth::ApiKeyGenerator;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/subscription", get(subscription).post(subscribe))
        .route("/subscription/plans", get(subscription_plans))
        .route("/usage", get(usage))
        .route("/keys", get(list_keys).post(create_key))
        .route("/keys/:key_id", delete(delete_key))
}

/// GET /v1/me/subscription
///
/// 获取当前用户的订阅信息
///
/// # 返回
/// - user_id: 用户ID
/// - email: 邮箱
/// - phone: 手机号
/// - subscription_plan: 订阅套餐名称
/// - subscription_start: 订阅开始时间
/// - subscription_end: 订阅结束时间
/// - is_active: 订阅是否有效
pub async fn subscription(
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = &auth.user;
    
    Ok(Json(serde_json::json!({
        "user_id": user.id.to_string(),
        "email": user.email,
        "phone": user.phone,
        "subscription_plan": user.subscription_plan.as_str(),
        "subscription_start": user.subscription_start,
        "subscription_end": user.subscription_end,
        "is_active": user.is_subscription_active(),
    })))
}

/// GET /v1/me/subscription/plans
///
/// 获取所有可用的订阅套餐列表
pub async fn subscription_plans() -> Result<Json<serde_json::Value>, ApiError> {
    let plans = SubscriptionPlanInfo::all();
    
    Ok(Json(serde_json::json!({
        "plans": plans
    })))
}

/// 订阅请求体
#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    /// 订阅套餐名称 (monthly, yearly, team, enterprise, zero_token)
    pub plan: String,
}

/// POST /v1/me/subscription
///
/// 订阅或更新用户的套餐
///
/// # 参数
/// * `Extension(auth)` - 认证上下文
/// * `Json(request)` - 订阅请求，包含要订阅的套餐名称
///
/// # 套餐持续时间
/// - monthly: 30天
/// - yearly: 365天
/// - team: 30天
/// - enterprise: 365天
/// - zero_token: 30天
pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<SubscribeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let plan = SubscriptionPlan::from_str(&request.plan);
    if plan == SubscriptionPlan::None {
        return Err(ApiError::InvalidRequest("Invalid subscription plan".to_string()));
    }

    // Calculate subscription duration
    let duration_days = match plan {
        SubscriptionPlan::Monthly => 30,
        SubscriptionPlan::Yearly => 365,
        SubscriptionPlan::Team => 30,
        SubscriptionPlan::Enterprise => 365,
        SubscriptionPlan::ZeroToken => 30,
        SubscriptionPlan::None => return Err(ApiError::InvalidRequest("Invalid plan".to_string())),
    };

    let now = chrono::Utc::now();
    let end = now + chrono::Duration::days(duration_days);

    // Update user subscription
    state.db.update_user_subscription(auth.user.id, plan, now, end)
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to update subscription")))?;

    Ok(Json(serde_json::json!({
        "message": "Subscription updated successfully",
        "plan": plan.as_str(),
        "subscription_start": now,
        "subscription_end": end,
    })))
}

/// GET /v1/me/usage
///
/// 获取用户在当前计费周期的使用统计
///
/// # 返回
/// - period_start: 计费周期开始时间
/// - period_end: 计费周期结束时间
/// - total_requests: 总请求数
/// - total_input_tokens: 输入 Token 总数
/// - total_output_tokens: 输出 Token 总数
/// - total_tokens: Token 总数
/// - token_quota: Token 配额上限
/// - quota_used_percent: 配额使用百分比
/// - avg_latency_ms: 平均延迟（毫秒）
/// - usage_by_provider: 按 Provider 分类的使用统计
/// - usage_by_model: 按 Model 分类的使用统计
pub async fn usage(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = &auth.user;

    // Determine billing period
    let now = chrono::Utc::now();
    let period_start = user.subscription_start.unwrap_or(now);
    let period_end = user.subscription_end.unwrap_or(
        now + chrono::Duration::days(user.subscription_plan.billing_cycle_days())
    );

    let stats = state.db
        .get_user_usage_stats(user.id, period_start, period_end)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to get usage stats: {}", e)))?;

    let quota = user.subscription_plan.monthly_token_quota();

    Ok(Json(serde_json::json!({
        "period_start": period_start,
        "period_end": period_end,
        "total_requests": stats.total_requests,
        "total_input_tokens": stats.total_input_tokens,
        "total_output_tokens": stats.total_output_tokens,
        "total_tokens": stats.total_input_tokens + stats.total_output_tokens,
        "token_quota": if quota == i64::MAX { serde_json::Value::Null } else { serde_json::json!(quota) },
        "quota_used_percent": if quota == i64::MAX { 0.0 } else {
            ((stats.total_input_tokens + stats.total_output_tokens) as f64 / quota as f64 * 100.0).min(100.0)
        },
        "avg_latency_ms": if stats.total_requests > 0 {
            stats.total_latency_ms / stats.total_requests
        } else {
            0
        },
        "usage_by_provider": stats.usage_by_provider.iter().map(|u| serde_json::json!({
            "provider": u.provider,
            "requests": u.requests,
            "input_tokens": u.input_tokens,
            "output_tokens": u.output_tokens,
        })).collect::<Vec<_>>(),
        "usage_by_model": stats.usage_by_model.iter().map(|u| serde_json::json!({
            "model": u.model,
            "provider": u.provider,
            "requests": u.requests,
            "input_tokens": u.input_tokens,
            "output_tokens": u.output_tokens,
        })).collect::<Vec<_>>(),
    })))
}

/// GET /v1/me/keys
///
/// 列出当前用户的所有 API Keys
///
/// # 返回
/// - id: Key ID
/// - name: Key 名称
/// - key_prefix: Key 前缀（用于显示）
/// - is_active: 是否激活
/// - last_used_at: 最后使用时间
/// - created_at: 创建时间
pub async fn list_keys(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let keys = state.db.list_api_keys_by_user(auth.user.id)
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to list API keys")))?;

    let keys_data: Vec<serde_json::Value> = keys.iter().map(|k| {
        serde_json::json!({
            "id": k.id.to_string(),
            "name": k.name,
            "key_prefix": k.key_prefix,
            "is_active": k.is_active,
            "last_used_at": k.last_used_at,
            "created_at": k.created_at,
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "data": keys_data
    })))
}

/// 创建 Key 请求体
#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    /// Key 名称（可选）
    pub name: Option<String>,
}

/// POST /v1/me/keys
///
/// 创建新的 API Key
///
/// # 注意
/// 返回的 plain_key 只显示一次，之后无法找回
///
/// # 返回
/// - id: Key ID
/// - key: 完整的 API Key（只显示一次）
/// - name: Key 名称
/// - created_at: 创建时间
pub async fn create_key(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<CreateKeyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let generator = ApiKeyGenerator::new("sk-nexus");
    let (plain_key, hashed_key) = generator.generate();

    let mut api_key = ApiKey::new(auth.user.id, hashed_key, format!("sk-nexus-{}", &plain_key[..15]));
    
    if let Some(name) = request.name {
        api_key = api_key.with_name(name);
    }

    state.db.create_api_key(&api_key)
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to create API key")))?;

    // Return the plain key only once - this is the only time it's shown
    Ok(Json(serde_json::json!({
        "id": api_key.id.to_string(),
        "key": plain_key,
        "name": api_key.name,
        "created_at": api_key.created_at,
    })))
}

/// DELETE /v1/me/keys/:key_id
///
/// 删除指定的 API Key
///
/// # 参数
/// * `Path(key_id)` - 要删除的 Key ID
pub async fn delete_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    axum::extract::Path(key_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key_uuid = uuid::Uuid::parse_str(&key_id)
        .map_err(|_| ApiError::InvalidRequest("Invalid key ID".to_string()))?;

    state.db.delete_api_key(key_uuid)
        .await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to delete API key")))?;

    Ok(Json(serde_json::json!({
        "deleted": true
    })))
}
