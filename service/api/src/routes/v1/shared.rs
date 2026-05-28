//! OpenAI 和 Anthropic 兼容路由共享工具模块
//!
//! 提供共享的功能：
//! - 速率限制计算
//! - Key 选择（含 BYOK 优先级）
//! - 余额检查
//! - 通用请求验证
//! - API 调用日志
//! - 模型列表实现

use axum::{http::HeaderMap, response::Response};
use billing::BillingEngine;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use crate::state::AppState;
use db;
use models::{CostBreakdown, ModelPricing, Provider, TokenCharge};
use provider_client::{HttpProviderClient, ProviderClient};
use router::key_scheduler::SelectedKey;

// ─── 速率限制常量 ────────────────────────────────────────────────────

const DEFAULT_RPM: i64 = 60;

/// 会话亲和性 Header 名称
pub const SESSION_HEADER: &str = "x-session-id";

/// 获取速率限制（统一默认值）
pub fn rate_limit_for_user() -> i64 {
    DEFAULT_RPM
}

/// 选择 API Key 进行请求（原接口，保留向后兼容）
pub async fn select_key(
    state: &Arc<AppState>,
    provider_slug: &str,
    session_id: Option<&str>,
) -> Result<Option<SelectedKey>, ApiError> {
    let mut scheduler = state.key_scheduler.write().await;
    scheduler.tick();
    match session_id {
        Some(sid) => Ok(scheduler.select_key_for_session(provider_slug, &sid)),
        None => Ok(scheduler.select_key_no_session(provider_slug)),
    }
}

// ─── Key 选择辅助函数 ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SelectedKeySource {
    UserKey {
        key_id: uuid::Uuid,
        provider_slug: String,
        decrypted_key: String,
        base_url: String,
    },
    SystemKey {
        selected: SelectedKey,
    },
}

/// 选择 API Key，优先使用用户自有 BYOK key
pub async fn select_key_with_priority(
    state: &Arc<AppState>,
    user_id: uuid::Uuid,
    provider_slug: &str,
    session_id: Option<&str>,
) -> Result<(SelectedKeySource, bool), ApiError> {
    // Step 1: 查找用户 Prioritized keys
    let user_keys = state
        .db
        .list_user_provider_keys_by_provider(user_id, provider_slug)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?;

    let prioritized: Vec<_> = user_keys
        .iter()
        .filter(|k| {
            k.is_active
                && k.priority_level == models::PriorityLevel::Prioritized
        })
        .collect();

    if !prioritized.is_empty() {
        let uk = prioritized[0];
        let decrypted = db::decrypt_api_key(&uk.api_key_encrypted)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Key decrypt failed: {}", e)))?;

        return Ok((
            SelectedKeySource::UserKey {
                key_id: uk.id,
                provider_slug: uk.provider_slug.clone(),
                decrypted_key: decrypted,
                base_url: uk.base_url.clone(),
            },
            true,
        ));
    }

    // Step 2: 回落系统 Key 池
    let selected = {
        let mut scheduler = state.key_scheduler.write().await;
        scheduler.tick();
        match session_id {
            Some(sid) => scheduler.select_key_for_session(provider_slug, sid),
            None => scheduler.select_key_no_session(provider_slug),
        }
    };

    match selected {
        Some(sk) => Ok((SelectedKeySource::SystemKey { selected: sk }, false)),
        None => {
            // Step 3: 尝试用户 Fallback keys
            let fallback: Vec<_> = user_keys
                .iter()
                .filter(|k| {
                    k.is_active
                        && k.priority_level == models::PriorityLevel::Fallback
                })
                .collect();

            if let Some(uk) = fallback.first() {
                let decrypted = db::decrypt_api_key(&uk.api_key_encrypted)
                    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Key decrypt failed: {}", e)))?;

                return Ok((
                    SelectedKeySource::UserKey {
                        key_id: uk.id,
                        provider_slug: uk.provider_slug.clone(),
                        decrypted_key: decrypted,
                        base_url: uk.base_url.clone(),
                    },
                    true,
                ));
            }

            Err(ApiError::InvalidRequest(format!(
                "No available key for provider: {}",
                provider_slug
            )))
        }
    }
}

/// 从 SelectedKeySource 创建 Provider HTTP 客户端
pub fn create_client_from_source(
    provider_slug: &str,
    source: &SelectedKeySource,
) -> Result<(Arc<dyn ProviderClient>, Option<uuid::Uuid>), ApiError> {
    match source {
        SelectedKeySource::UserKey {
            decrypted_key,
            base_url,
            key_id,
            ..
        } => {
            let final_url = if base_url.is_empty() {
                provider_slug.to_string()
            } else {
                base_url.clone()
            };
            let client = HttpProviderClient::new_with_decrypted_key(
                &final_url,
                decrypted_key,
                *key_id,
            )
            .map_err(|e| ApiError::ProviderError(format!("Failed to create client: {}", e)))?;
            Ok((Arc::new(client), Some(*key_id)))
        }
        SelectedKeySource::SystemKey { selected } => {
            let decrypted_key =
                db::decrypt_api_key(&selected.key.api_key_encrypted).map_err(|e| {
                    ApiError::Internal(anyhow::anyhow!("Failed to decrypt provider key: {}", e))
                })?;
            let client = HttpProviderClient::new_with_decrypted_key(
                provider_slug,
                &decrypted_key,
                selected.key.id,
            )
            .map_err(|e| ApiError::ProviderError(format!("Failed to create client: {}", e)))?;
            Ok((Arc::new(client), Some(selected.key.id)))
        }
    }
}

/// 创建 Provider HTTP 客户端（保留向后兼容）
pub fn create_client(
    provider_slug: &str,
    selected: Option<SelectedKey>,
) -> Result<(Arc<dyn ProviderClient>, Option<uuid::Uuid>), ApiError> {
    match selected {
        Some(sk) => {
            let decrypted_key = db::decrypt_api_key(&sk.key.api_key_encrypted).map_err(|e| {
                ApiError::Internal(anyhow::anyhow!("Failed to decrypt provider key: {}", e))
            })?;
            let client = HttpProviderClient::new_with_decrypted_key(
                provider_slug,
                &decrypted_key,
                sk.key.id,
            )
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

/// 记录请求结果
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

// ─── 会话辅助函数 ─────────────────────────────────────────────────────────

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

// ─── 余额检查 ─────────────────────────────────────────────────────

/// 检查用户余额是否足够
pub async fn check_balance(state: &AppState, user_id: uuid::Uuid) -> Result<(), ApiError> {
    let balance = state
        .db
        .get_user_balance(user_id)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Balance check failed: {}", e)))?;

    if balance.balance <= Decimal::ZERO {
        return Err(ApiError::InvalidRequest(
            "Insufficient balance. Please top up your credits.".to_string(),
        ));
    }
    Ok(())
}

// ─── 计费流水线 ────────────────────────────────────────────────

/// 计算并扣费
pub async fn charge_tokens(
    state: &Arc<AppState>,
    user_id: uuid::Uuid,
    generation_id: uuid::Uuid,
    model_slug: &str,
    provider_slug: &str,
    api_log_id: uuid::Uuid,
    is_free: bool,
    provider_key_id: Option<uuid::Uuid>,
    user_provider_key_id: Option<uuid::Uuid>,
    input_tokens: i32,
    output_tokens: i32,
    reasoning_tokens: i32,
    image_count: i32,
    cache_read_tokens: i32,
) -> (CostBreakdown, String) {
    let key_source = if is_free {
        if user_provider_key_id.is_some() {
            "user"
        } else {
            "system_free"
        }
    } else {
        "system"
    };

    if is_free {
        let breakdown = CostBreakdown::default();
        let charge = TokenCharge {
            id: uuid::Uuid::new_v4(),
            user_id,
            api_log_id: Some(api_log_id),
            generation_id,
            key_source: key_source.to_string(),
            user_provider_key_id,
            provider_key_id,
            model_slug: model_slug.to_string(),
            provider_slug: provider_slug.to_string(),
            input_tokens,
            output_tokens,
            reasoning_tokens,
            image_count,
            cache_read_tokens,
            is_free: true,
            created_at: chrono::Utc::now(),
            ..Default::default()
        };
        let _ = state.db.create_token_charge(&charge).await;
        return (breakdown, key_source.to_string());
    }

    match state.db.get_model_pricing(model_slug).await {
        Ok(Some(pricing)) => {
            let breakdown = BillingEngine::calculate(
                &pricing,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                image_count,
                cache_read_tokens,
            );

            // 扣减余额
            if let Err(e) = state.db.deduct_balance(user_id, breakdown.total).await {
                tracing::warn!("Failed to deduct balance for user {}: {:?}", user_id, e);
            }

            let charge = TokenCharge {
                id: uuid::Uuid::new_v4(),
                user_id,
                api_log_id: Some(api_log_id),
                generation_id,
                key_source: key_source.to_string(),
                user_provider_key_id,
                provider_key_id,
                model_slug: model_slug.to_string(),
                provider_slug: provider_slug.to_string(),
                input_tokens,
                output_tokens,
                reasoning_tokens,
                image_count,
                cache_read_tokens,
                prompt_cost: breakdown.prompt_cost,
                completion_cost: breakdown.completion_cost,
                image_cost: breakdown.image_cost,
                total_cost: breakdown.total,
                is_free: false,
                created_at: chrono::Utc::now(),
                ..Default::default()
            };
            let _ = state.db.create_token_charge(&charge).await;

            (breakdown, key_source.to_string())
        }
        Ok(None) => {
            tracing::warn!("No pricing found for model: {}", model_slug);
            (CostBreakdown::default(), key_source.to_string())
        }
        Err(e) => {
            tracing::error!("Failed to get model pricing: {:?}", e);
            (CostBreakdown::default(), key_source.to_string())
        }
    }
}

// ─── 验证辅助函数 ─────────────────────────────────────────────────────

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
    fn hv(v: &str) -> HeaderValue {
        v.parse().unwrap()
    }
    headers.insert("X-RateLimit-Limit", hv(&limit.to_string()));
    headers.insert("X-RateLimit-Remaining", hv(&remaining.to_string()));
    headers.insert("X-RateLimit-Reset", hv(&reset.to_string()));
}

// ─── API 日志记录 ─────────────────────────────────────────────────────────────

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

// ─── 模型列表实现 ────────────────────────────────────────────────────

pub async fn list_models_impl(state: &Arc<AppState>) -> Result<Vec<serde_json::Value>, ApiError> {
    if let Ok(Some(cached)) = state.redis.get_cached_models().await {
        if let Ok(all_models) = serde_json::from_str::<Vec<models::ModelWithProvider>>(&cached) {
            return Ok(all_models
                .iter()
                .map(serde_json::to_value)
                .filter_map(|r| r.ok())
                .collect());
        }
    }

    let providers = state
        .db
        .list_providers()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to load providers: {}", e)))?;

    let provider_map: std::collections::HashMap<String, &Provider> =
        providers.iter().map(|p| (p.slug.clone(), p)).collect();

    let all_models = state
        .db
        .list_models()
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to load models: {}", e)))?;

    let pricing_list = state
        .db
        .list_model_pricing()
        .await
        .unwrap_or_default();
    let pricing_map: std::collections::HashMap<String, &ModelPricing> =
        pricing_list.iter().map(|p| (p.model_slug.clone(), p)).collect();

    let models_with_providers: Vec<serde_json::Value> = all_models
        .iter()
        .filter_map(|m| {
            let provider = provider_map.get(&m.provider_id)?;
            let pricing = pricing_map.get(&m.slug);
            let mut entry = serde_json::json!({
                "id":              m.slug.clone(),
                "object":          "model",
                "created":         1713123123,
                "owned_by":        m.provider_id.clone(),
                "permission":      [],
                "root":            m.slug.clone(),
                "parent":          serde_json::Value::Null,
                "provider":        m.provider_id.clone(),
                "provider_name":   provider.name.clone(),
                "context_window":  m.context_window,
                "capabilities":    m.capabilities,
            });
            if let Some(p) = pricing {
                entry["pricing"] = serde_json::json!({
                    "prompt": p.prompt_price,
                    "completion": p.completion_price,
                    "image": p.image_price,
                    "reasoning": p.reasoning_price,
                    "cache_read": p.cache_read_price,
                    "request": p.request_price,
                    "mode": p.pricing_mode.as_str(),
                });
            }
            Some(entry)
        })
        .collect();

    if let Ok(json) = serde_json::to_string(&models_with_providers) {
        let _ = state.redis.cache_models(&json).await;
    }

    Ok(models_with_providers)
}
