//! Shared utilities used by both OpenAI- and Anthropic-compatible routes.
//!
//! Contains rate-limit helpers, key selection, validation, and the shared
//! model listing implementation.

use std::sync::Arc;
use axum::{
    extract::State,
    http::HeaderMap,
    response::Response,
    Json, Extension,
};
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use futures_util::StreamExt;

use crate::error::ApiError;
use crate::state::AppState;
use crate::middleware::auth::AuthContext;
use models::{Message as InternalMessage, User, SubscriptionPlan, ModelWithProvider, Provider};
use provider_client::{ProviderClient, HttpProviderClient};
use router::key_scheduler::SelectedKey;

// ─── Rate-limit constants ────────────────────────────────────────────────────

const FREE_RPM: i64 = 10;
const MONTHLY_RPM: i64 = 60;
const YEARLY_RPM: i64 = 120;
const TEAM_RPM: i64 = 300;
const ENTERPRISE_RPM: i64 = 1000;

pub const SESSION_HEADER: &str = "x-session-id";

pub fn rate_limit_for_plan(plan: &SubscriptionPlan) -> i64 {
    match plan {
        SubscriptionPlan::None => FREE_RPM,
        SubscriptionPlan::Monthly => MONTHLY_RPM,
        SubscriptionPlan::Yearly => YEARLY_RPM,
        SubscriptionPlan::Team => TEAM_RPM,
        SubscriptionPlan::Enterprise => ENTERPRISE_RPM,
    }
}

// ─── Key selection helpers ───────────────────────────────────────────────────

pub async fn select_key(
    state: &Arc<AppState>,
    provider_slug: &str,
    session_id: Option<&str>,
) -> Result<Option<SelectedKey>, ApiError> {
    let mut scheduler = state.key_scheduler.write().await;
    scheduler.tick();
    match session_id {
        Some(sid) => Ok(scheduler.select_key_for_session(provider_slug, sid)),
        None => Ok(scheduler.select_key_no_session(provider_slug)),
    }
}

pub fn create_client(
    provider_slug: &str,
    selected: Option<SelectedKey>,
) -> Result<(Arc<dyn ProviderClient>, Option<uuid::Uuid>), ApiError> {
    match selected {
        Some(sk) => {
            let client = HttpProviderClient::new_with_key(provider_slug, &sk.key)
                .map_err(|e| ApiError::ProviderError(format!("Failed to create client: {}", e)))?;
            Ok((Arc::new(client), Some(sk.key.id)))
        }
        None => {
            let client = HttpProviderClient::new(provider_slug)
                .map_err(|e| ApiError::ProviderError(format!("Failed to create client: {}", e)))?;
            Ok((Arc::new(client), None))
        }
    }
}

pub async fn record_result(
    state: &Arc<AppState>,
    provider_slug: &str,
    key_id: Option<uuid::Uuid>,
    latency_ms: i32,
    success: bool,
) {
    if let Some(kid) = key_id {
        let mut scheduler = state.key_scheduler.write().await;
        if success {
            scheduler.record_success(provider_slug, kid, latency_ms as f64);
        } else {
            scheduler.record_failure(provider_slug, kid);
        }
    }
}

// ─── Session helpers ─────────────────────────────────────────────────────────

pub fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get(SESSION_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

pub fn default_session_id(auth: &AuthContext) -> Option<String> {
    Some(auth.user.id.to_string())
}

// ─── Validation helpers ─────────────────────────────────────────────────────

pub fn check_subscription(user: &User) -> Result<(), ApiError> {
    match user.subscription_plan {
        SubscriptionPlan::None => {}
        SubscriptionPlan::Monthly | SubscriptionPlan::Yearly |
        SubscriptionPlan::Team | SubscriptionPlan::Enterprise => {
            if let (Some(start), Some(end)) = (user.subscription_start, user.subscription_end) {
                let now = chrono::Utc::now();
                if now < start || now > end {
                    return Err(ApiError::SubscriptionExpired);
                }
            } else {
                return Err(ApiError::SubscriptionExpired);
            }
        }
    }
    Ok(())
}

pub async fn check_token_quota(state: &AppState, user: &User) -> Result<(), ApiError> {
    let quota = user.subscription_plan.monthly_token_quota();
    if quota == i64::MAX {
        return Ok(());
    }

    let now = chrono::Utc::now();
    let period_start = user.subscription_start.unwrap_or(now);
    let period_end = user.subscription_end.unwrap_or(
        now + chrono::Duration::days(user.subscription_plan.billing_cycle_days()),
    );

    if now > period_end {
        return Err(ApiError::SubscriptionExpired);
    }

    let used = state
        .db
        .get_user_token_usage_in_period(user.id, period_start, period_end)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Token quota check failed: {}", e)))?;

    if used >= quota {
        return Err(ApiError::InvalidRequest(format!(
            "Token quota exceeded. Used {} / {}.", used, quota
        )));
    }
    Ok(())
}

pub fn validate_temperature(temperature: f32) -> Result<(), ApiError> {
    if !(0.0..=2.0).contains(&temperature) {
        return Err(ApiError::InvalidRequest(
            "temperature must be between 0.0 and 2.0".to_string(),
        ));
    }
    Ok(())
}

pub fn add_rate_limit_headers(response: &mut Response, limit: i64, remaining: i64, reset: i64) {
    use axum::http::HeaderValue;
    let headers = response.headers_mut();
    fn hv(v: &str) -> HeaderValue { v.parse().unwrap() }
    headers.insert("X-RateLimit-Limit", hv(&limit.to_string()));
    headers.insert("X-RateLimit-Remaining", hv(&remaining.to_string()));
    headers.insert("X-RateLimit-Reset", hv(&reset.to_string()));
}

// ─── API logging ─────────────────────────────────────────────────────────────

pub async fn log_api_call(
    state: &AppState,
    auth: &AuthContext,
    provider_id: &str,
    model_id: &str,
    mode: &str,
    input_tokens: i32,
    output_tokens: i32,
    latency_ms: i32,
) {
    let log = models::ApiLog::new(
        auth.user.id,
        auth.api_key_id.unwrap_or(uuid::Uuid::nil()),
        provider_id.to_string(),
        model_id.to_string(),
        mode.to_string(),
    )
    .with_tokens(input_tokens, output_tokens)
    .with_latency(latency_ms);

    if let Err(e) = state.db.create_api_log(&log).await {
        tracing::error!("Failed to create API log: {}", e);
    }
}

// ─── Model listing helper ────────────────────────────────────────────────────

pub async fn list_models_impl(
    state: &Arc<AppState>,
) -> Result<Vec<serde_json::Value>, ApiError> {
    // Try cache first
    if let Ok(Some(cached)) = state.redis.get_cached_models().await {
        if let Ok(all_models) = serde_json::from_str::<Vec<ModelWithProvider>>(&cached) {
            let result: Vec<serde_json::Value> = all_models
                .iter()
                .map(serde_json::to_value)
                .filter_map(|r| r.ok())
                .collect();
            return Ok(result);
        }
    }

    let providers = state
        .db
        .list_providers()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to load providers: {}", e)))?;

    let provider_map: std::collections::HashMap<String, &Provider> = providers
        .iter()
        .map(|p| (p.slug.clone(), p))
        .collect();

    let all_models = state
        .db
        .list_models()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to load models: {}", e)))?;

    let models_with_providers: Vec<serde_json::Value> = all_models
        .iter()
        .filter_map(|m| {
            let provider = provider_map.get(&m.provider_id)?;
            let provider_name = provider.name.clone();
            Some(serde_json::json!({
                "id":              m.slug.clone(),
                "object":          "model",
                "created":         1713123123, // placeholder
                "owned_by":        m.provider_id.clone(),
                "permission":      [],
                "root":            m.slug.clone(),
                "parent":          serde_json::Value::Null,
                "provider":        m.provider_id.clone(),
                "provider_name":   provider_name,
                "context_window":  m.context_window,
                "capabilities":    m.capabilities,
            }))
        })
        .collect();

    // Cache result
    if let Ok(json) = serde_json::to_string(&models_with_providers) {
        let _ = state.redis.cache_models(&json).await;
    }

    Ok(models_with_providers)
}
