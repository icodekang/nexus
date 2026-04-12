use axum::{
    routing::{get, post, put, delete},
    Router, Json, Extension,
    extract::{Query, Path, State},
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use models::{Provider, LlmModel, ModelMode};
use db::postgres::DashboardStats;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/dashboard/stats", get(dashboard_stats))
        .route("/users", get(list_users))
        .route("/users/:id", put(update_user))
        .route("/providers", get(list_providers).post(create_provider))
        .route("/providers/:id", put(update_provider).delete(delete_provider))
        .route("/models", get(list_models).post(create_model))
        .route("/models/:id", put(update_model).delete(delete_model))
        .route("/transactions", get(list_transactions))
}

// ============ Dashboard ============

async fn dashboard_stats(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<DashboardStats>, ApiError> {
    let stats = state.db.get_dashboard_stats().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to get dashboard stats: {}", e)))?;

    Ok(Json(stats))
}

// ============ Users ============

#[derive(Debug, Deserialize)]
struct UserQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    search: Option<String>,
}

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

#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    phone: Option<String>,
    subscription_plan: Option<String>,
}

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

// ============ Providers ============

#[derive(Debug, Deserialize)]
struct CreateProviderRequest {
    name: String,
    slug: String,
    api_base_url: Option<String>,
    priority: Option<i32>,
}

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

#[derive(Debug, Deserialize)]
struct UpdateProviderRequest {
    name: Option<String>,
    slug: Option<String>,
    api_base_url: Option<String>,
    is_active: Option<bool>,
    priority: Option<i32>,
}

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

// ============ Models ============

#[derive(Debug, Deserialize)]
struct CreateModelRequest {
    provider_id: String,
    name: String,
    slug: String,
    model_id: String,
    mode: Option<String>,
    context_window: Option<i32>,
    capabilities: Option<Vec<String>>,
}

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

#[derive(Debug, Deserialize)]
struct UpdateModelRequest {
    name: Option<String>,
    slug: Option<String>,
    model_id: Option<String>,
    context_window: Option<i32>,
    capabilities: Option<Vec<String>>,
    is_active: Option<bool>,
}

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
        body.context_window,
        caps_json,
        body.is_active,
    ).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update model: {}", e)))?;

    Ok(Json(serde_json::json!({ "updated": true })))
}

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
