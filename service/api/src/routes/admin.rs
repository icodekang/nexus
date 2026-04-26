//! 管理后台路由模块
//! 提供管理员操作接口，包括用户管理、Provider 管理、Model 管理和交易记录查询
//!
//! 路由列表：
//! - GET /dashboard/stats - 获取仪表盘统计数据
//! - GET /users - 用户列表
//! - PUT /users/:id - 更新用户信息
//! - GET /providers - Provider 列表
//! - POST /providers - 创建 Provider
//! - PUT /providers/:id - 更新 Provider
//! - DELETE /providers/:id - 删除 Provider
//! - GET /provider-keys - Provider Keys 列表
//! - POST /provider-keys - 创建 Provider Key
//! - PUT /provider-keys/:id - 更新 Provider Key
//! - DELETE /provider-keys/:id - 删除 Provider Key
//! - POST /provider-keys/:id/test - 测试 Provider Key
//! - GET /models - Model 列表
//! - POST /models - 创建 Model
//! - PUT /models/:id - 更新 Model
//! - DELETE /models/:id - 删除 Model
//! - GET /transactions - 交易记录列表
//! - /accounts/* - 浏览器账户管理（嵌套路由）

use axum::{
    routing::{get, post, put, delete},
    Router, Json, Extension,
    extract::{Query, Path, State},
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::state::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use models::{Provider, ProviderKey, LlmModel, ModelMode};
use db::postgres::{DashboardStats, RevenueTrend, RecentActivity};

pub mod accounts;

/// 创建管理后台路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/dashboard/stats", get(dashboard_stats))
        .route("/dashboard/revenue-trends", get(revenue_trends))
        .route("/dashboard/recent-activity", get(recent_activity))
        .route("/users", get(list_users))
        .route("/users/:id", put(update_user))
        .route("/providers", get(list_providers).post(create_provider))
        .route("/providers/:id", put(update_provider).delete(delete_provider))
        .route("/provider-keys", get(list_provider_keys).post(create_provider_key))
        .route("/provider-keys/:id", put(update_provider_key).delete(delete_provider_key))
        .route("/provider-keys/:id/test", post(test_provider_key))
        .route("/user-keys", get(list_user_api_keys))
        .route("/user-keys/:id", delete(delete_user_api_key))
        .route("/models", get(list_models).post(create_model))
        .route("/models/:id", put(update_model).delete(delete_model))
        .route("/transactions", get(list_transactions))
        .merge(accounts::routes())
}

// ============ 仪表盘 ============

/// GET /admin/dashboard/stats
///
/// 获取仪表盘统计数据，包括用户数、收入、活跃用户等
async fn dashboard_stats(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<DashboardStats>, ApiError> {
    let stats = state.db.get_dashboard_stats().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to get dashboard stats: {}", e)))?;

    Ok(Json(stats))
}

/// GET /admin/dashboard/revenue-trends
///
/// 获取每日收入趋势数据
async fn revenue_trends(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Query(params): Query<RevenueTrendsQuery>,
) -> Result<Json<Vec<RevenueTrend>>, ApiError> {
    let days = params.days.unwrap_or(30).clamp(7, 90);
    let trends = state.db.get_revenue_trends(days).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to get revenue trends: {}", e)))?;
    Ok(Json(trends))
}

/// GET /admin/dashboard/recent-activity
///
/// 获取最近活动列表
async fn recent_activity(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<Vec<RecentActivity>>, ApiError> {
    let activities = state.db.get_recent_activity(10).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to get recent activity: {}", e)))?;
    Ok(Json(activities))
}

/// 收入趋势查询参数
#[derive(Debug, Deserialize)]
struct RevenueTrendsQuery {
    days: Option<i32>,
}

/// 用户查询参数
#[derive(Debug, Deserialize)]
struct UserQuery {
    /// 页码（从1开始）
    page: Option<u64>,
    /// 每页数量（最大100）
    per_page: Option<u64>,
    /// 搜索关键字（搜索邮箱）
    search: Option<String>,
}

/// 用户列表项
#[derive(Debug, Serialize)]
struct UserListItem {
    id: String,
    email: String,
    phone: Option<String>,
    subscription_plan: String,
    is_admin: bool,
    is_active: bool,
    created_at: String,
    updated_at: String,
}

async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Query(query): Query<UserQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let search = query.search.unwrap_or_default();
    let offset = (page - 1) * per_page;

    let users = state.db.list_users(offset as i64, per_page as i64, &search).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list users: {}", e)))?;

    let total = state.db.count_users(&search).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to count users: {}", e)))?;

    let data: Vec<UserListItem> = users.into_iter().map(|u| {
        let is_active = u.is_subscription_active();
        UserListItem {
            id: u.id.to_string(),
            email: u.email,
            phone: u.phone,
            subscription_plan: u.subscription_plan.as_str().to_string(),
            is_admin: u.is_admin,
            is_active,
            created_at: u.created_at.to_rfc3339(),
            updated_at: u.updated_at.to_rfc3339(),
        }
    }).collect();

    Ok(Json(serde_json::json!({
        "data": data,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}

/// 更新用户请求体
#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    /// 新手机号（可选）
    phone: Option<String>,
    /// 新订阅套餐（可选）
    subscription_plan: Option<String>,
}

/// PUT /admin/users/:id
///
/// 更新用户信息（管理员操作）
async fn update_user(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid user ID".to_string()))?;

    state.db.update_user_admin(
        user_id,
        body.phone.as_deref(),
        body.subscription_plan.as_deref(),
    ).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update user: {}", e)))?;

    // Return updated user
    let user = state.db.get_user_by_id(user_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("User not found".to_string()))?;

    Ok(Json(serde_json::json!({
        "id": user.id.to_string(),
        "email": user.email,
        "phone": user.phone,
        "subscription_plan": user.subscription_plan.as_str(),
        "is_admin": user.is_admin,
        "is_active": user.is_subscription_active(),
        "created_at": user.created_at.to_rfc3339(),
        "updated_at": user.updated_at.to_rfc3339(),
    })))
}

// ============ Provider 管理 ============

/// 创建 Provider 请求体
#[derive(Debug, Deserialize)]
struct CreateProviderRequest {
    /// Provider 名称
    name: String,
    /// Provider slug（唯一标识）
    slug: String,
    /// API 基础 URL（可选）
    api_base_url: Option<String>,
    /// 优先级（可选，数字越大优先级越高）
    priority: Option<i32>,
}

/// GET /admin/providers
///
/// 获取所有 Provider 列表
async fn list_providers(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let providers = state.db.list_all_providers().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list providers: {}", e)))?;

    let data: Vec<serde_json::Value> = providers.iter().map(|p| {
        serde_json::json!({
            "id": p.id.to_string(),
            "name": p.name,
            "slug": p.slug,
            "logo_url": p.logo_url,
            "api_base_url": p.api_base_url,
            "is_active": p.is_active,
            "priority": p.priority,
            "created_at": p.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

/// POST /admin/providers
///
/// 创建新的 Provider
async fn create_provider(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Json(body): Json<CreateProviderRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut provider = Provider::new(
        body.name,
        body.slug,
        body.api_base_url.unwrap_or_else(|| "https://api.example.com/v1".to_string()),
    );
    if let Some(p) = body.priority {
        provider = provider.with_priority(p);
    }

    state.db.create_provider(&provider).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create provider: {}", e)))?;

    Ok(Json(serde_json::json!({
        "id": provider.id.to_string(),
        "name": provider.name,
        "slug": provider.slug,
        "api_base_url": provider.api_base_url,
        "is_active": provider.is_active,
        "priority": provider.priority,
        "created_at": provider.created_at.to_rfc3339(),
    })))
}

/// 更新 Provider 请求体
#[derive(Debug, Deserialize)]
struct UpdateProviderRequest {
    name: Option<String>,
    slug: Option<String>,
    api_base_url: Option<String>,
    is_active: Option<bool>,
    priority: Option<i32>,
}

/// PUT /admin/providers/:id
///
/// 更新 Provider 信息
async fn update_provider(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<UpdateProviderRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let provider_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid provider ID".to_string()))?;

    state.db.update_provider(
        provider_id,
        body.name.as_deref(),
        body.slug.as_deref(),
        body.api_base_url.as_deref(),
        body.is_active,
        body.priority,
    ).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update provider: {}", e)))?;

    Ok(Json(serde_json::json!({ "updated": true })))
}

/// DELETE /admin/providers/:id
///
/// 软删除 Provider（将 is_active 设为 false）
async fn delete_provider(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let provider_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid provider ID".to_string()))?;

    state.db.delete_provider_soft(provider_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to delete provider: {}", e)))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============ Model 管理 ============

/// 创建 Model 请求体
#[derive(Debug, Deserialize)]
struct CreateModelRequest {
    /// 所属 Provider ID
    provider_id: String,
    /// Model 名称
    name: String,
    /// Model slug（唯一标识）
    slug: String,
    /// 实际模型 ID（如 gpt-4o）
    model_id: String,
    /// 模式：chat、completion、embedding
    mode: Option<String>,
    /// 上下文窗口大小
    context_window: Option<i32>,
    /// 支持的能力列表
    capabilities: Option<Vec<String>>,
}

/// GET /admin/models
///
/// 获取所有 Model 列表
async fn list_models(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let models = state.db.list_all_models().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list models: {}", e)))?;

    let data: Vec<serde_json::Value> = models.iter().map(|m| {
        serde_json::json!({
            "id": m.id.to_string(),
            "provider_id": m.provider_id,
            "name": m.name,
            "slug": m.slug,
            "model_id": m.model_id,
            "mode": m.mode.as_str(),
            "context_window": m.context_window,
            "capabilities": m.capabilities,
            "is_active": m.is_active,
            "created_at": m.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

/// POST /admin/models
///
/// 创建新的 Model
async fn create_model(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Json(body): Json<CreateModelRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mode = match body.mode.as_deref() {
        Some("completion") => ModelMode::Completion,
        Some("embedding") => ModelMode::Embedding,
        _ => ModelMode::Chat,
    };

    // Check for duplicate slug
    let slug_exists = state.db.model_slug_exists(&body.slug).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("DB error: {}", e)))?;

    if slug_exists {
        return Err(ApiError::InvalidRequest(format!("Model with slug '{}' already exists", body.slug)));
    }

    let model = LlmModel::new(
        body.provider_id,
        body.name,
        body.slug,
        body.model_id,
        mode,
        body.context_window.unwrap_or(4096),
    ).with_capabilities(body.capabilities.unwrap_or_default());

    state.db.create_model(&model).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create model: {}", e)))?;

    Ok(Json(serde_json::json!({
        "id": model.id.to_string(),
        "provider_id": model.provider_id,
        "name": model.name,
        "slug": model.slug,
        "model_id": model.model_id,
        "mode": model.mode.as_str(),
        "context_window": model.context_window,
        "capabilities": model.capabilities,
        "is_active": model.is_active,
        "created_at": model.created_at.to_rfc3339(),
    })))
}

/// 更新 Model 请求体
#[derive(Debug, Deserialize)]
struct UpdateModelRequest {
    name: Option<String>,
    slug: Option<String>,
    model_id: Option<String>,
    provider_id: Option<String>,
    context_window: Option<i32>,
    capabilities: Option<Vec<String>>,
    is_active: Option<bool>,
}

/// PUT /admin/models/:id
///
/// 更新 Model 信息
async fn update_model(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<UpdateModelRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let model_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid model ID".to_string()))?;

    let caps_json = body.capabilities.map(|c| serde_json::to_value(&c).unwrap());

    state.db.update_model(
        model_id,
        body.name.as_deref(),
        body.slug.as_deref(),
        body.model_id.as_deref(),
        body.provider_id.as_deref(),
        body.context_window,
        caps_json,
        body.is_active,
    ).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update model: {}", e)))?;

    Ok(Json(serde_json::json!({ "updated": true })))
}

/// DELETE /admin/models/:id
///
/// 软删除 Model（将 is_active 设为 false）
async fn delete_model(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let model_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid model ID".to_string()))?;

    state.db.delete_model_soft(model_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to delete model: {}", e)))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============ Provider Keys ============

/// Encode API key (base64 only, no encryption)
fn encode_api_key(key: &str) -> String {
    BASE64.encode(key.as_bytes())
}

/// Decode API key (base64 only, no encryption)
fn decode_api_key(encoded: &str) -> Result<String, ApiError> {
    let decoded = BASE64.decode(encoded)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to decode API key: invalid base64")))?;
    String::from_utf8(decoded)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to decode API key: invalid UTF-8")))
}

fn extract_key_prefix(key: &str) -> String {
    if key.len() <= 12 {
        key.to_string()
    } else {
        key[..12].to_string()
    }
}

#[derive(Debug, Deserialize)]
struct CreateProviderKeyRequest {
    provider_slug: String,
    api_key: String,
    base_url: Option<String>,
    priority: Option<i32>,
}

async fn list_provider_keys(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let keys = state.db.list_provider_keys().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list provider keys: {}", e)))?;

    let data: Vec<serde_json::Value> = keys.iter().map(|k| {
        let preview = decode_api_key(&k.api_key_encrypted)
            .unwrap_or_default();
        serde_json::json!({
            "id": k.id.to_string(),
            "provider_slug": k.provider_slug,
            "api_key_masked": k.masked_key(),
            "api_key_preview": if preview.len() > 8 {
                format!("{}...{}", &preview[..4], &preview[preview.len()-4..])
            } else {
                preview
            },
            "base_url": k.base_url,
            "is_active": k.is_active,
            "priority": k.priority,
            "created_at": k.created_at.to_rfc3339(),
            "updated_at": k.updated_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

async fn create_provider_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Json(body): Json<CreateProviderKeyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let encrypted = encode_api_key(&body.api_key);
    let prefix = extract_key_prefix(&body.api_key);

    let mut key = ProviderKey::new(
        body.provider_slug,
        encrypted,
        prefix,
        body.base_url.unwrap_or_default(),
    );
    if let Some(p) = body.priority {
        key = key.with_priority(p);
    }

    state.db.create_provider_key(&key).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create provider key: {}", e)))?;

    Ok(Json(serde_json::json!({
        "id": key.id.to_string(),
        "provider_slug": key.provider_slug,
        "api_key_masked": key.masked_key(),
        "api_key_preview": "",
        "base_url": key.base_url,
        "is_active": key.is_active,
        "priority": key.priority,
        "created_at": key.created_at.to_rfc3339(),
        "updated_at": key.updated_at.to_rfc3339(),
    })))
}

#[derive(Debug, Deserialize)]
struct UpdateProviderKeyRequest {
    api_key: Option<String>,
    base_url: Option<String>,
    is_active: Option<bool>,
    priority: Option<i32>,
}

async fn update_provider_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<UpdateProviderKeyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid provider key ID".to_string()))?;

    let (encrypted, prefix) = match &body.api_key {
        Some(k) => (Some(encode_api_key(k)), Some(extract_key_prefix(k))),
        None => (None, None),
    };

    state.db.update_provider_key(
        key_id,
        encrypted.as_deref(),
        prefix.as_deref(),
        body.base_url.as_deref(),
        body.is_active,
        body.priority,
    ).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update provider key: {}", e)))?;

    Ok(Json(serde_json::json!({ "updated": true })))
}

async fn delete_provider_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid provider key ID".to_string()))?;

    state.db.delete_provider_key(key_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to delete provider key: {}", e)))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn test_provider_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid provider key ID".to_string()))?;

    let provider_key = state.db.get_provider_key_by_id(key_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("Provider key not found".to_string()))?;

    let api_key = decode_api_key(&provider_key.api_key_encrypted)?;

    // Test by making a minimal request to the provider's /models endpoint
    let client = reqwest::Client::new();
    let test_url = if provider_key.base_url.is_empty() {
        format!("https://api.example.com/v1/models")
    } else {
        format!("{}/models", provider_key.base_url.trim_end_matches('/'))
    };

    let result = client.get(&test_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match result {
        Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 401 || resp.status().as_u16() == 403 => {
            // 401/403 means the key was recognized (just may lack permissions for /models)
            // This still proves the key is valid and the endpoint is reachable
            let success = resp.status().is_success() || resp.status().as_u16() == 403;
            Ok(Json(serde_json::json!({
                "success": success,
                "message": if success { "Connection successful" } else { "Invalid API key" }
            })))
        }
        Ok(resp) => {
            Ok(Json(serde_json::json!({
                "success": false,
                "message": format!("Unexpected response: {}", resp.status())
            })))
        }
        Err(e) => {
            Ok(Json(serde_json::json!({
                "success": false,
                "message": format!("Connection failed: {}", e)
            })))
        }
    }
}

// ============ User API Keys ============

#[derive(Debug, Deserialize)]
struct UserApiKeyQuery {
    user_id: Option<String>,
    user_email: Option<String>,
}

async fn list_user_api_keys(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Query(query): Query<UserApiKeyQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let keys = state.db.list_all_api_keys_with_users().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list user API keys: {}", e)))?;

    let data: Vec<serde_json::Value> = keys.iter().filter(|(k, email)| {
        let matches_user_id = query.user_id.as_ref().map_or(true, |uid| k.user_id.to_string() == *uid);
        let matches_email = query.user_email.as_ref().map_or(true, |em| email.contains(em));
        matches_user_id && matches_email
    }).map(|(k, email)| {
        serde_json::json!({
            "id": k.id.to_string(),
            "user_id": k.user_id.to_string(),
            "user_email": email,
            "name": k.name,
            "key_prefix": k.key_prefix,
            "is_active": k.is_active,
            "last_used_at": k.last_used_at,
            "created_at": k.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

async fn delete_user_api_key(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid key ID".to_string()))?;

    state.db.delete_api_key(key_id).await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to delete API key")))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============ Transactions ============

#[derive(Debug, Deserialize)]
struct TransactionQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    #[serde(rename = "type")]
    tx_type: Option<String>,
    status: Option<String>,
}

async fn list_transactions(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Query(query): Query<TransactionQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let tx_type = query.tx_type.unwrap_or_default();
    let status = query.status.unwrap_or_default();
    let offset = (page - 1) * per_page;

    let rows = state.db.list_all_transactions(offset as i64, per_page as i64, &tx_type, &status).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list transactions: {}", e)))?;

    let total = state.db.count_all_transactions(&tx_type, &status).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to count transactions: {}", e)))?;

    let data: Vec<serde_json::Value> = rows.iter().map(|(tx, email)| {
        serde_json::json!({
            "id": tx.id.to_string(),
            "user_id": tx.user_id.to_string(),
            "user_email": email,
            "transaction_type": format!("{:?}", tx.transaction_type).to_lowercase(),
            "amount": tx.amount,
            "plan": tx.plan.as_ref().map(|p| p.as_str()),
            "status": format!("{:?}", tx.status).to_lowercase(),
            "description": tx.description,
            "created_at": tx.created_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(serde_json::json!({
        "data": data,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}
