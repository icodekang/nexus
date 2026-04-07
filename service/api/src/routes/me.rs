use axum::{routing::{get, post, delete}, Router, Json, Extension};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use models::{ApiKey, subscription::SubscriptionPlanInfo};
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
pub async fn subscription(
    State(state): State<Arc<AppState>>,
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
pub async fn subscription_plans() -> Result<Json<serde_json::Value>, ApiError> {
    let plans = SubscriptionPlanInfo::all();
    
    Ok(Json(serde_json::json!({
        "plans": plans
    })))
}

/// Subscribe request body
#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub plan: String,
}

/// POST /v1/me/subscription
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
pub async fn usage(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // In a real implementation, we would query the api_logs table
    // and aggregate usage statistics
    
    Ok(Json(serde_json::json!({
        "total_requests": 0,
        "total_input_tokens": 0,
        "total_output_tokens": 0,
        "usage_by_model": {},
        "usage_by_provider": {},
    })))
}

/// GET /v1/me/keys
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

/// Create key request
#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: Option<String>,
}

/// POST /v1/me/keys
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
pub async fn delete_key(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
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
