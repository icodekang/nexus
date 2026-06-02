use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use crate::state::AppState;
use auth::ApiKeyGenerator;
use db;
use models::{ApiKey, NexusKeyType, PriorityLevel, UserProviderKey};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/balance", get(get_balance))
        .route("/usage", get(usage))
        .route("/usage/daily", get(daily_usage))
        .route("/usage/daily/by-model", get(daily_usage_by_model))
        .route("/charges", get(list_charges))
        .route("/packages", get(list_packages))
        .route("/purchase", post(purchase_package))
        .route("/recharge", post(recharge))
        .route("/transactions", get(list_transactions))
        .route("/keys", get(list_keys).post(create_key))
        .route("/keys/:key_id", delete(delete_key))
        .route("/provider-keys", get(list_provider_keys).post(create_provider_key))
        .route("/provider-keys/:key_id", put(update_provider_key).delete(delete_provider_key))
}

// ─── 余额 ───────────────────────────────────────────────────────────

/// GET /v1/me/balance
async fn get_balance(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.db.ensure_user_balance(auth.user.id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    let balance = state.db.get_user_balance(auth.user.id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({
        "balance": balance.balance,
        "total_purchased": balance.total_purchased,
        "total_consumed": balance.total_consumed,
    })))
}

// ─── 使用统计 ───────────────────────────────────────────────────────

/// GET /v1/me/usage
async fn usage(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = chrono::Utc::now();
    let period_start = now - chrono::Duration::days(30);

    let stats = state
        .db
        .get_user_usage_stats(auth.user.id, period_start, now)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to get usage stats: {}", e)))?;

    let balance = state.db.get_user_balance(auth.user.id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({
        "period_start": period_start,
        "period_end": now,
        "total_requests": stats.total_requests,
        "total_input_tokens": stats.total_input_tokens,
        "total_output_tokens": stats.total_output_tokens,
        "total_tokens": stats.total_input_tokens + stats.total_output_tokens,
        "balance": balance.balance,
        "total_consumed": balance.total_consumed,
        "avg_latency_ms": if stats.total_requests > 0 {
            stats.total_latency_ms / stats.total_requests
        } else { 0 },
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

/// GET /v1/me/usage/daily - 最近7天每日消费
async fn daily_usage(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let days = 7;
    let charges = state
        .db
        .get_daily_charges(auth.user.id, days)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to get daily charges: {}", e)))?;

    let mut filled: Vec<serde_json::Value> = Vec::new();
    for i in (0..days).rev() {
        let date = chrono::Utc::now().date_naive() - chrono::Duration::days(i as i64);
        let charge = charges.iter().find(|c| c.day == date);
        filled.push(serde_json::json!({
            "day": date.format("%Y-%m-%d").to_string(),
            "input_tokens": charge.map(|c| c.input_tokens).unwrap_or(0),
            "output_tokens": charge.map(|c| c.output_tokens).unwrap_or(0),
            "total_cost": charge.map(|c| c.total_cost).unwrap_or(rust_decimal::Decimal::ZERO),
        }));
    }

    Ok(Json(serde_json::json!({ "data": filled })))
}

/// GET /v1/me/usage/daily/by-model - 最近7天每日按模型分组消费
async fn daily_usage_by_model(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let days = 7;
    let charges = state
        .db
        .get_daily_charges_by_model(auth.user.id, days)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to get daily model charges: {}", e)))?;

    let mut result: Vec<serde_json::Value> = Vec::new();
    for i in (0..days).rev() {
        let date = chrono::Utc::now().date_naive() - chrono::Duration::days(i as i64);
        let day_str = date.format("%Y-%m-%d").to_string();
        let day_charges: Vec<_> = charges.iter().filter(|c| c.day == date).collect();
        let models: Vec<_> = day_charges.iter().map(|c| serde_json::json!({
            "model": c.model_slug,
            "cost": c.total_cost,
        })).collect();
        let total: rust_decimal::Decimal = day_charges.iter().map(|c| c.total_cost).sum();
        result.push(serde_json::json!({
            "day": day_str,
            "models": models,
            "total_cost": total,
        }));
    }

    Ok(Json(serde_json::json!({ "data": result })))
}

// ─── 消费明细 ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ChargesQuery {
    page: Option<i64>,
    per_page: Option<i64>,
}

/// GET /v1/me/charges
async fn list_charges(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    axum::extract::Query(query): axum::extract::Query<ChargesQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let charges = state.db.list_token_charges(auth.user.id, per_page, offset).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({
        "data": charges.iter().map(|c| serde_json::json!({
            "id": c.id,
            "model": c.model_slug,
            "provider": c.provider_slug,
            "input_tokens": c.input_tokens,
            "output_tokens": c.output_tokens,
            "total_cost": c.total_cost,
            "is_free": c.is_free,
            "key_source": c.key_source,
            "created_at": c.created_at,
        })).collect::<Vec<_>>(),
        "page": page,
        "per_page": per_page,
    })))
}

// ─── 套餐 ───────────────────────────────────────────────────────────

/// GET /v1/me/packages
async fn list_packages(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let packages = state.db.list_token_packages().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({
        "packages": packages.iter().map(|p| serde_json::json!({
            "id": p.id,
            "name": p.name,
            "credits": p.credits,
            "price": p.price,
            "bonus_credits": p.bonus_credits,
        })).collect::<Vec<_>>(),
    })))
}

#[derive(Debug, Deserialize)]
struct PurchaseRequest {
    package_id: String,
}

/// POST /v1/me/purchase
async fn purchase_package(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<PurchaseRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pkg_id = uuid::Uuid::parse_str(&req.package_id)
        .map_err(|_| ApiError::InvalidRequest("Invalid package ID".to_string()))?;

    let packages = state.db.list_token_packages().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    let pkg = packages.iter().find(|p| p.id == pkg_id)
        .ok_or(ApiError::InvalidRequest("Package not found".to_string()))?;

    let total = pkg.credits + pkg.bonus_credits;
    state.db.add_balance(auth.user.id, total).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    let tx = models::Transaction::new(
        auth.user.id,
        models::TransactionType::TokenPurchase,
        pkg.price.to_string().parse().unwrap_or(0.0),
    )
    .with_plan(pkg.name.clone())
    .with_description(format!("Purchased {} credits ({} bonus)", pkg.credits, pkg.bonus_credits));
    let _ = state.db.create_transaction(&tx).await;

    let balance = state.db.get_user_balance(auth.user.id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Purchase successful",
        "credits_added": total,
        "balance": balance.balance,
    })))
}

#[derive(Debug, Deserialize)]
struct RechargeRequest {
    amount: f64,
}

/// POST /v1/me/recharge — 直接充值（跳过支付）
async fn recharge(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<RechargeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if req.amount <= 0.0 {
        return Err(ApiError::InvalidRequest("Amount must be positive".to_string()));
    }
    if req.amount > 10000.0 {
        return Err(ApiError::InvalidRequest("Amount exceeds maximum".to_string()));
    }

    let amount = rust_decimal::Decimal::from_f64_retain(req.amount)
        .unwrap_or(rust_decimal::Decimal::ZERO);
    state.db.add_balance(auth.user.id, amount).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    let tx = models::Transaction::new(
        auth.user.id,
        models::TransactionType::TokenPurchase,
        req.amount,
    )
    .with_plan("直接充值".to_string())
    .with_description(format!("Recharged ${:.2}", req.amount));
    let _ = state.db.create_transaction(&tx).await;

    let balance = state.db.get_user_balance(auth.user.id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "Recharge successful",
        "amount": req.amount,
        "balance": balance.balance,
    })))
}

/// GET /v1/me/transactions
async fn list_transactions(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let txs = state
        .db
        .list_transactions(auth.user.id, 50)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({
        "data": txs.iter().map(|tx| serde_json::json!({
            "id": tx.id,
            "type": tx.transaction_type.as_str(),
            "amount": tx.amount,
            "plan": tx.plan,
            "status": tx.status.as_str(),
            "description": tx.description,
            "created_at": tx.created_at.to_rfc3339(),
        })).collect::<Vec<_>>(),
    })))
}

// ─── Nexus API Keys ────────────────────────────────────────────────

/// GET /v1/me/keys
async fn list_keys(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let keys = state.db.list_api_keys_by_user(auth.user.id).await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to list API keys")))?;

    Ok(Json(serde_json::json!({
        "data": keys.iter().map(|k| serde_json::json!({
            "id": k.id,
            "name": k.name,
            "key_prefix": k.key_prefix,
            "is_active": k.is_active,
            "last_used_at": k.last_used_at,
            "created_at": k.created_at,
            "key_type": k.key_type.as_str(),
        })).collect::<Vec<_>>(),
    })))
}

#[derive(Debug, Deserialize)]
struct CreateKeyRequest {
    pub name: Option<String>,
    pub key_type: Option<String>,
}

/// POST /v1/me/keys
async fn create_key(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<CreateKeyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let generator = ApiKeyGenerator::new("sk-nexus");
    let (plain_key, hashed_key) = generator.generate();

    let key_type = match request.key_type.as_deref() {
        Some("openai_sdk") => NexusKeyType::OpenAiSdk,
        Some("anthropic_sdk") => NexusKeyType::AnthropicSdk,
        _ => NexusKeyType::HttpMessages,
    };

    let mut api_key = ApiKey::new(
        auth.user.id,
        hashed_key,
        format!("sk-nexus-{}", &plain_key[9..20]),
    )
    .with_key_type(key_type);
    if let Some(name) = request.name {
        api_key = api_key.with_name(name);
    }

    state.db.create_api_key(&api_key).await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to create API key")))?;

    Ok(Json(serde_json::json!({
        "id": api_key.id,
        "key": plain_key,
        "name": api_key.name,
        "key_type": api_key.key_type.as_str(),
        "created_at": api_key.created_at,
    })))
}

/// DELETE /v1/me/keys/:key_id
async fn delete_key(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Path(key_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key_uuid = uuid::Uuid::parse_str(&key_id)
        .map_err(|_| ApiError::InvalidRequest("Invalid key ID".to_string()))?;

    let keys = state.db.list_api_keys_by_user(auth.user.id).await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to list API keys")))?;

    if !keys.iter().any(|k| k.id == key_uuid) {
        return Err(ApiError::Forbidden);
    }

    state.db.delete_api_key(key_uuid).await
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to delete API key")))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ─── BYOK Provider Keys ──────────────────────────────────────────

/// GET /v1/me/provider-keys
async fn list_provider_keys(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let keys = state.db.list_user_provider_keys(auth.user.id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({
        "data": keys.iter().map(|k| serde_json::json!({
            "id": k.id,
            "provider_slug": k.provider_slug,
            "name": k.name,
            "api_key_prefix": k.mask_key(),
            "is_active": k.is_active,
            "priority_level": k.priority_level.as_str(),
            "sort_order": k.sort_order,
            "always_use": k.always_use,
            "created_at": k.created_at,
        })).collect::<Vec<_>>(),
    })))
}

#[derive(Debug, Deserialize)]
struct CreateProviderKeyRequest {
    provider_slug: String,
    api_key: String,
    name: Option<String>,
    base_url: Option<String>,
    priority_level: Option<String>,
    always_use: Option<bool>,
}

/// POST /v1/me/provider-keys
async fn create_provider_key(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateProviderKeyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let encrypted = db::encrypt_api_key(&req.api_key);

    let prefix = if req.api_key.len() > 12 {
        format!("{}...{}", &req.api_key[..6], &req.api_key[req.api_key.len()-6..])
    } else {
        req.api_key.clone()
    };

    let mut key = UserProviderKey::new(
        auth.user.id,
        req.provider_slug.clone(),
        encrypted,
        prefix,
        req.base_url.unwrap_or_default(),
    );

    if let Some(n) = req.name {
        key = key.with_name(n);
    }

    if let Some(pl) = req.priority_level {
        key.priority_level = PriorityLevel::from_str(&pl);
    }
    if let Some(au) = req.always_use {
        key.always_use = au;
    }

    state.db.create_user_provider_key(&key).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({
        "id": key.id,
        "provider_slug": key.provider_slug,
        "name": key.name,
        "created_at": key.created_at,
    })))
}

#[derive(Debug, Deserialize)]
struct UpdateProviderKeyRequest {
    name: Option<String>,
    is_active: Option<bool>,
    priority_level: Option<String>,
    sort_order: Option<i32>,
    always_use: Option<bool>,
}

/// PUT /v1/me/provider-keys/:key_id
async fn update_provider_key(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Path(key_id): Path<String>,
    Json(req): Json<UpdateProviderKeyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key_uuid = uuid::Uuid::parse_str(&key_id)
        .map_err(|_| ApiError::InvalidRequest("Invalid key ID".to_string()))?;

    state.db.update_user_provider_key(
        key_uuid,
        auth.user.id,
        req.name.as_deref(),
        req.is_active,
        req.priority_level.as_deref(),
        req.sort_order,
        req.always_use,
        None, // model_filter
    ).await.map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({ "updated": true })))
}

/// DELETE /v1/me/provider-keys/:key_id
async fn delete_provider_key(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Path(key_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key_uuid = uuid::Uuid::parse_str(&key_id)
        .map_err(|_| ApiError::InvalidRequest("Invalid key ID".to_string()))?;

    state.db.delete_user_provider_key(key_uuid, auth.user.id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
