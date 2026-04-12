//! OpenAI- and Anthropic-compatible API endpoints.
//!
//! Both interfaces are adapted to the internal `ProviderClient` interface so they
//! share the same key-scheduler, fallback, rate-limiting and logging logic.
//!
//! ## OpenAI-compatible  (`/v1/openai/*`)
//! Request/response shape matches the OpenAI Chat Completions API exactly.
//!
//! ## Anthropic-compatible (`/v1/anthropic/*`)
//! Request/response shape matches the Anthropic Messages API.
//! Key differences from OpenAI:
//!   - `max_tokens` is required (Anthropic always generates until limit)
//!   - System prompt is sent as a `user` message with `\n\nHuman: ` prefix
//!   - Streaming uses SSE `event:` lines (`message_start`, `content_block_delta`, …)
//!   - Response body is `{"id":"...","content":[{"type":"text","text":"..."}]}`

use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderName},
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json, Extension,
};
use std::sync::Arc;
use futures_util::{StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;
use crate::middleware::auth::AuthContext;
use models::{ChatRequest as InternalChatRequest, ChatResponse as InternalChatResponse,
             Message as InternalMessage, Usage, User, SubscriptionPlan, ChatChunk};
use provider_client::{
    ChatRequest as ProviderChatRequest,
    Message as ProviderMessage,
    ProviderClient, HttpProviderClient,
};
use router::key_scheduler::SelectedKey;

const SESSION_HEADER: &str = "x-session-id";

// ─── Rate-limit constants ────────────────────────────────────────────────────

const FREE_RPM: i64 = 10;
const MONTHLY_RPM: i64 = 60;
const YEARLY_RPM: i64 = 120;
const TEAM_RPM: i64 = 300;
const ENTERPRISE_RPM: i64 = 1000;

fn rate_limit_for_plan(plan: &SubscriptionPlan) -> i64 {
    match plan {
        SubscriptionPlan::None => FREE_RPM,
        SubscriptionPlan::Monthly => MONTHLY_RPM,
        SubscriptionPlan::Yearly => YEARLY_RPM,
        SubscriptionPlan::Team => TEAM_RPM,
        SubscriptionPlan::Enterprise => ENTERPRISE_RPM,
    }
}

// ─── Unified internal chat handler ─────────────────────────────────────────

/// Shared logic used by both OpenAI and Anthropic routes.
/// Accepts already-parsed messages in the internal format.
/// `provider_slug` is the provider to route to (e.g. "openai", "anthropic").
/// `stream` controls whether to use SSE streaming.
async fn unified_chat(
    state: Arc<AppState>,
    auth: AuthContext,
    provider_slug: String,
    model: String,
    messages: Vec<InternalMessage>,
    temperature: f32,
    max_tokens: Option<i32>,
    is_stream: bool,
    session_id: Option<String>,
) -> Result<Response, ApiError> {
    let user_id = auth.user.id.to_string();
    let rpm = rate_limit_for_plan(&auth.user.subscription_plan);

    // Rate limit
    let (allowed, remaining, reset_time) = state.redis
        .check_rate_limit(&user_id, rpm, 60)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Rate limit check failed: {}", e)))?;

    if !allowed {
        return Err(ApiError::RateLimitExceeded);
    }

    // Subscription check
    check_subscription(&auth.user)?;

    // Token quota
    check_token_quota(&state, &auth.user).await?;

    // Look up model
    let db_model = state.db.get_model_by_slug(&model)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?
        .ok_or_else(|| ApiError::ModelNotFound(model.clone()))?;

    // Validate
    validate_temperature(temperature)?;
    if let Some(mt) = max_tokens {
        if mt <= 0 || mt > db_model.context_window {
            return Err(ApiError::InvalidRequest(
                format!("max_tokens ({}) out of range", mt)
            ));
        }
    }

    let provider_for_request = if provider_slug.is_empty() {
        db_model.provider_id.clone()
    } else {
        provider_slug.clone()
    };

    // Build provider request
    let provider_messages: Vec<ProviderMessage> = messages.iter().map(|m| {
        ProviderMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        }
    }).collect();

    let provider_request = ProviderChatRequest {
        provider: provider_for_request.clone(),
        model: db_model.model_id.clone(),
        messages: provider_messages,
        temperature,
        max_tokens,
        stream: is_stream,
        extra: std::collections::HashMap::new(),
    };

    let start_time = std::time::Instant::now();

    if is_stream {
        stream_handler(state, auth, provider_slug, model, start_time, rpm, remaining, reset_time,
                       provider_for_request, provider_request).await
    } else {
        blocking_handler(state, auth, provider_slug, model, start_time, rpm, remaining, reset_time,
                         provider_for_request, provider_request).await
    }
}

// ─── Non-streaming handler ──────────────────────────────────────────────────

async fn blocking_handler(
    state: Arc<AppState>,
    auth: AuthContext,
    _provider_slug: String,
    model_name: String,
    start_time: std::time::Instant,
    rpm: i64,
    remaining: i64,
    reset_time: i64,
    provider_for_request: String,
    mut provider_request: ProviderChatRequest,
) -> Result<Response, ApiError> {
    let mut last_error = None;
    let mut provider_resp = None;
    let mut used_key_id: Option<uuid::Uuid> = None;

    // Primary provider with session affinity
    let selected_key = select_key(&state, &provider_for_request, session_id(&state, &auth).as_deref()).await?;
    let (client, key_id) = create_client(&provider_for_request, selected_key)?;

    provider_request.provider = provider_for_request.clone();
    match client.chat(provider_request.clone()).await {
        Ok(resp) => {
            provider_resp = Some(resp);
            used_key_id = key_id;
            let latency_ms = start_time.elapsed().as_millis() as i32;
            record_result(&state, &provider_for_request, key_id, latency_ms, true).await;
        }
        Err(e) => {
            tracing::warn!("Primary provider {} failed: {}", provider_for_request, e);
            last_error = Some(e);
            record_result(&state, &provider_for_request, key_id, 0, false).await;
        }
    }

    // Fallback providers
    if provider_resp.is_none() {
        if let Ok(all_providers) = state.db.list_providers().await {
            for p in &all_providers {
                if p.is_active && p.slug != provider_for_request {
                    let selected_key = select_key(&state, &p.slug, None).await?;
                    let (client, key_id) = create_client(&p.slug, selected_key)?;

                    provider_request.provider = p.slug.clone();
                    match client.chat(provider_request.clone()).await {
                        Ok(resp) => {
                            provider_resp = Some(resp);
                            used_key_id = key_id;
                            let latency_ms = start_time.elapsed().as_millis() as i32;
                            record_result(&state, &p.slug, key_id, latency_ms, true).await;
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("Fallback provider {} failed: {}", p.slug, e);
                            last_error = Some(e);
                            record_result(&state, &p.slug, key_id, 0, false).await;
                        }
                    }
                }
            }
        }
    }

    let provider_resp = provider_resp.ok_or_else(|| {
        ApiError::ProviderError(format!("All providers failed. Last error: {:?}", last_error))
    })?;

    let latency_ms = start_time.elapsed().as_millis() as i32;
    let prompt_tokens = provider_resp.usage.get("prompt_tokens").copied().unwrap_or(0);
    let completion_tokens = provider_resp.usage.get("completion_tokens").copied().unwrap_or(0);
    let total_tokens = provider_resp.usage.get("total_tokens").copied()
        .unwrap_or(prompt_tokens + completion_tokens);

    log_api_call(&state, &auth, &provider_for_request, &model_name,
                 "chat", prompt_tokens, completion_tokens, latency_ms).await;

    let mut http_response = Json(provider_resp).into_response();
    add_rate_limit_headers(&mut http_response, rpm, remaining, reset_time);
    Ok(http_response)
}

// ─── Streaming handler ─────────────────────────────────────────────────────

async fn stream_handler(
    state: Arc<AppState>,
    auth: AuthContext,
    provider_slug: String,
    model_name: String,
    start_time: std::time::Instant,
    rpm: i64,
    remaining: i64,
    reset_time: i64,
    provider_for_request: String,
    provider_request: ProviderChatRequest,
) -> Result<Response, ApiError> {
    let (tx, rx) = broadcast::channel::<Result<Event, std::convert::Infallible>>(1024);
    let state_clone = state.clone();
    let auth_clone = auth.clone();
    let provider_clone = provider_for_request.clone();
    let model_clone = model_name.clone();
    let sid = session_id(&state, &auth);

    tokio::spawn(async move {
        let mut used_key_id: Option<uuid::Uuid> = None;
        let selected_key = {
            let mut scheduler = state_clone.key_scheduler.write().await;
            scheduler.tick();
            sid.as_ref().map(|s| scheduler.select_key_for_session(&provider_clone, s)).flatten()
        };

        let client_result = match &selected_key {
            Some(sk) => {
                used_key_id = Some(sk.key.id);
                HttpProviderClient::new_with_key(&provider_clone, &sk.key)
            }
            None => HttpProviderClient::new(&provider_clone),
        };

        match client_result {
            Ok(client) => {
                match client.chat_stream(provider_request).await {
                    Ok(chunks) => {
                        for chunk in chunks {
                            let event = if chunk.finished {
                                let chat_chunk = ChatChunk::new(&model_name, "", true);
                                Event::default()
                                    .event("message")
                                    .data(serde_json::to_string(&chat_chunk).unwrap_or_default())
                            } else {
                                let chat_chunk = ChatChunk::new(&model_name, &chunk.delta, false);
                                Event::default()
                                    .event("message")
                                    .data(serde_json::to_string(&chat_chunk).unwrap_or_default())
                            };
                            let _ = tx.send(Ok(event));
                            if chunk.finished { break; }
                        }
                    }
                    Err(e) => {
                        if let Some(kid) = used_key_id {
                            let mut s = state_clone.key_scheduler.write().await;
                            s.record_failure(&provider_clone, kid);
                        }
                        let _ = tx.send(Ok(Event::default()
                            .event("error")
                            .data(format!("{{\"error\": \"{}\"}}", e))));
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(format!("{{\"error\": \"Failed to create client: {}\"}}", e))));
            }
        }

        if let Some(kid) = used_key_id {
            let latency_ms = start_time.elapsed().as_millis() as i32;
            let mut scheduler = state_clone.key_scheduler.write().await;
            scheduler.record_success(&provider_clone, kid, latency_ms as f64);
        }

        let latency_ms = start_time.elapsed().as_millis() as i32;
        log_api_call(&state_clone, &auth_clone, &provider_clone, &model_clone,
                     "chat", 0, 0, latency_ms).await;
    });

    let stream = BroadcastStream::new(rx).filter_map(|item| async {
        match item {
            Ok(Ok(event)) => Some(Ok::<_, std::convert::Infallible>(event)),
            _ => None,
        }
    });

    let mut response = Sse::new(stream).into_response();
    add_rate_limit_headers(&mut response, rpm, remaining, reset_time);
    Ok(response)
}

// ─── Key selection helpers ───────────────────────────────────────────────────

async fn select_key(
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

fn create_client(
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

async fn record_result(
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

fn session_id(state: &Arc<AppState>, auth: &AuthContext) -> Option<String> {
    // In a real implementation we'd extract x-session-id from the caller.
    // For now every authenticated user gets user-level affinity.
    Some(auth.user.id.to_string())
}

// ─── Validation helpers ─────────────────────────────────────────────────────

fn check_subscription(user: &User) -> Result<(), ApiError> {
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

async fn check_token_quota(state: &AppState, user: &User) -> Result<(), ApiError> {
    let quota = user.subscription_plan.monthly_token_quota();
    if quota == i64::MAX { return Ok(()); }

    let now = chrono::Utc::now();
    let period_start = user.subscription_start.unwrap_or(now);
    let period_end = user.subscription_end.unwrap_or(
        now + chrono::Duration::days(user.subscription_plan.billing_cycle_days())
    );

    if now > period_end { return Err(ApiError::SubscriptionExpired); }

    let used = state.db
        .get_user_token_usage_in_period(user.id, period_start, period_end)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Token quota check failed: {}", e)))?;

    if used >= quota {
        return Err(ApiError::InvalidRequest(
            format!("Token quota exceeded. Used {} / {}.", used, quota)
        ));
    }
    Ok(())
}

fn validate_temperature(temperature: f32) -> Result<(), ApiError> {
    if !(0.0..=2.0).contains(&temperature) {
        return Err(ApiError::InvalidRequest("temperature must be between 0.0 and 2.0".to_string()));
    }
    Ok(())
}

fn add_rate_limit_headers(response: &mut Response, limit: i64, remaining: i64, reset: i64) {
    use axum::http::{HeaderValue, HeaderName};
    let headers = response.headers_mut();
    fn hv(v: &str) -> HeaderValue { v.parse().unwrap() }
    headers.insert("X-RateLimit-Limit", hv(&limit.to_string()));
    headers.insert("X-RateLimit-Remaining", hv(&remaining.to_string()));
    headers.insert("X-RateLimit-Reset", hv(&reset.to_string()));
}

async fn log_api_call(
    state: &AppState,
    auth: &AuthContext,
    provider_id: &str,
    model_id: &str,
    mode: &str,
    input_tokens: i32,
    output_tokens: i32,
    latency_ms: i32,
) {
    use uuid::Uuid;
    let log = models::ApiLog::new(
        auth.user.id,
        auth.api_key_id.unwrap_or(Uuid::nil()),
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

// ─────────────────────────────────────────────────────────────────────────────
// OpenAI-compatible endpoints  (`/v1/openai/*`)
// ─────────────────────────────────────────────────────────────────────────────

/// OpenAI-style query parameters
#[derive(Debug, Deserialize)]
pub struct OpenAIQuery {
    pub stream: Option<bool>,
}

/// POST /v1/openai/chat/completions
pub async fn openai_chat_completions(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Query(_query): Query<OpenAIQuery>,
    Json(request): Json<OpenAIClientRequest>,
) -> Result<Response, ApiError> {
    let is_stream = request.stream.unwrap_or(false);

    // Parse "provider/model" or just "model"
    let (provider_slug, model_slug) = parse_model_string(&request.model);

    let session_id = extract_session_id(&headers)
        .or_else(|| Some(auth.user.id.to_string()));

    unified_chat(
        state, auth, provider_slug, model_slug,
        request.messages.into_iter().map(Into::into).collect(),
        request.temperature.unwrap_or(0.7),
        request.max_tokens,
        is_stream,
        session_id,
    ).await
}

#[derive(Debug, Deserialize)]
pub struct OpenAIClientRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<i32>,
    #[serde(rename = "stream", default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub name: Option<String>,
}

impl From<OpenAIMessage> for InternalMessage {
    fn from(m: OpenAIMessage) -> Self {
        InternalMessage { role: m.role, content: m.content, name: m.name }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Anthropic-compatible endpoints  (`/v1/anthropic/*`)
// ─────────────────────────────────────────────────────────────────────────────

/// Anthropic-style streaming query parameters
#[derive(Debug, Deserialize)]
pub struct AnthropicQuery {
    #[serde(default)]
    pub stream: Option<bool>,
}

/// POST /v1/anthropic/messages
pub async fn anthropic_messages(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Query(_query): Query<AnthropicQuery>,
    Json(request): Json<AnthropicRequest>,
) -> Result<Response, ApiError> {
    // Anthropic always streams via SSE when stream=true
    let is_stream = request.stream.unwrap_or(false);

    // Transform Anthropic messages → internal format.
    // Anthropic uses "user" and "assistant" roles; system is embedded in the
    // first user message as "\n\nHuman: " prefix (Anthropic convention).
    let internal_messages: Vec<InternalMessage> = request.messages.into_iter().map(|m| {
        match m.role.as_str() {
            // Pass through user/assistant directly
            "user" | "assistant" => InternalMessage {
                role: m.role,
                content: m.content,
                name: None,
            },
            // Anthropic sends system as a separate message with role "user" and
            // content prefixed with "This is a system prompt: ..." — we just pass it through
            // as a "system" role so the router understands it.
            "system" => InternalMessage {
                role: "system".to_string(),
                content: m.content,
                name: None,
            },
            _ => InternalMessage {
                role: m.role,
                content: m.content,
                name: None,
            },
        }
    }).collect();

    // Anthropic model names are bare ("claude-3-5-sonnet-20241022")
    // Map them to our internal model slugs if needed.
    let model = request.model.clone();

    // max_tokens is required by Anthropic
    let max_tokens = request.max_tokens;

    // Anthropic uses beta header for versioned APIs — we ignore it here.
    let sid = extract_session_id(&headers)
        .or_else(|| Some(auth.user.id.to_string()));

    if is_stream {
        anthropic_stream_handler(state, auth, model, internal_messages,
                                 request.temperature.unwrap_or(1.0),
                                 max_tokens, sid).await
    } else {
        anthropic_blocking_handler(state, auth, model, internal_messages,
                                   request.temperature.unwrap_or(1.0),
                                   max_tokens, sid).await
    }
}

#[derive(Debug, Deserialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    #[serde(default)]
    pub max_tokens: Option<i32>,
    #[serde(rename = "temperature", default)]
    pub temperature: Option<f32>,
    /// Anthropic streaming — enabled via stream=true query param or body field.
    #[serde(rename = "stream", default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub system: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

// ─── Anthropic non-streaming ────────────────────────────────────────────────

async fn anthropic_blocking_handler(
    state: Arc<AppState>,
    auth: AuthContext,
    model: String,
    messages: Vec<InternalMessage>,
    temperature: f32,
    max_tokens: Option<i32>,
    session_id: Option<String>,
) -> Result<Response, ApiError> {
    // Inject system prompt into first user message if present
    let messages = prepend_system(messages, ()).0;

    let user_id = auth.user.id.to_string();
    let rpm = rate_limit_for_plan(&auth.user.subscription_plan);

    let (allowed, remaining, reset_time) = state.redis
        .check_rate_limit(&user_id, rpm, 60)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Rate limit check failed: {}", e)))?;

    if !allowed { return Err(ApiError::RateLimitExceeded); }
    check_subscription(&auth.user)?;
    check_token_quota(&state, &auth.user).await?;

    let provider_slug = "anthropic".to_string();

    let provider_request = ProviderChatRequest {
        provider: provider_slug.clone(),
        model: model.clone(),
        messages: messages.iter().map(|m| ProviderMessage {
            role: m.role.clone(), content: m.content.clone()
        }).collect(),
        temperature,
        max_tokens,
        stream: false,
        extra: std::collections::HashMap::new(),
    };

    let start_time = std::time::Instant::now();
    let selected_key = select_key(&state, &provider_slug, session_id.as_deref()).await?;
    let (client, key_id) = create_client(&provider_slug, selected_key)?;

    let resp = client.chat(provider_request).await
        .map_err(|e| ApiError::ProviderError(format!("Provider error: {}", e)))?;

    if let Some(kid) = key_id {
        let mut scheduler = state.key_scheduler.write().await;
        scheduler.record_success(&provider_slug, kid, start_time.elapsed().as_millis() as f64);
    }

    let latency_ms = start_time.elapsed().as_millis() as i32;
    let prompt_tokens = resp.usage.get("prompt_tokens").copied().unwrap_or(0);
    let completion_tokens = resp.usage.get("completion_tokens").copied().unwrap_or(0);

    log_api_call(&state, &auth, &provider_slug, &model, "chat",
                 prompt_tokens, completion_tokens, latency_ms).await;

    // Transform internal response → Anthropic format
    let anthropic_resp = AnthropicResponse {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        #[allow(deprecated)]
        r#type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![AnthropicContentBlock::Text {
            text: resp.message.content,
            #[allow(deprecated)]
            r#type: "text".to_string(),
        }],
        model: model.clone(),
        stop_reason: Some("end_turn".to_string()),
        stop_sequence: None,
        usage: AnthropicUsage {
            input_tokens: prompt_tokens,
            output_tokens: completion_tokens,
        },
    };

    let mut http_response = Json(anthropic_resp).into_response();
    add_rate_limit_headers(&mut http_response, rpm, remaining, reset_time);
    Ok(http_response)
}

// ─── Anthropic streaming ────────────────────────────────────────────────────

async fn anthropic_stream_handler(
    state: Arc<AppState>,
    auth: AuthContext,
    model: String,
    messages: Vec<InternalMessage>,
    temperature: f32,
    max_tokens: Option<i32>,
    session_id: Option<String>,
) -> Result<Response, ApiError> {
    let (tx, rx) = broadcast::channel::<Result<Event, std::convert::Infallible>>(1024);
    let state_clone = state.clone();
    let auth_clone = auth.clone();
    let provider_slug = "anthropic".to_string();

    tokio::spawn(async move {
        let mut used_key_id: Option<uuid::Uuid> = None;

        let selected_key = {
            let mut scheduler = state_clone.key_scheduler.write().await;
            scheduler.tick();
            session_id.as_ref().map(|s| scheduler.select_key_for_session(&provider_slug, s)).flatten()
        };

        let client_result = match &selected_key {
            Some(sk) => {
                used_key_id = Some(sk.key.id);
                HttpProviderClient::new_with_key(&provider_slug, &sk.key)
            }
            None => HttpProviderClient::new(&provider_slug),
        };

        let start_time = std::time::Instant::now();

        match client_result {
            Ok(client) => {
                let provider_request = ProviderChatRequest {
                    provider: provider_slug.clone(),
                    model: model.clone(),
                    messages: messages.iter().map(|m| ProviderMessage {
                        role: m.role.clone(), content: m.content.clone()
                    }).collect(),
                    temperature,
                    max_tokens,
                    stream: true,
                    extra: std::collections::HashMap::new(),
                };

                match client.chat_stream(provider_request).await {
                    Ok(chunks) => {
                        // Send message_start event
                        let _ = tx.send(Ok(Event::default()
                            .event("message_start")
                            .data(serde_json::json!({
                                "type": "message_start",
                                "message": {
                                    "id": format!("msg_{}", uuid::Uuid::new_v4()),
                                    "type": "message",
                                    "role": "assistant",
                                    "content": [],
                                    "model": model.clone(),
                                }
                            }).to_string())));

                        // Send content_block_start
                        let _ = tx.send(Ok(Event::default()
                            .event("content_block_start")
                            .data(serde_json::json!({
                                "type": "content_block_start",
                                "index": 0,
                                "content_block": {
                                    "type": "text",
                                    "text": "",
                                }
                            }).to_string())));

                        let mut text_accum = String::new();

                        for chunk in chunks {
                            if !chunk.delta.is_empty() {
                                text_accum.push_str(&chunk.delta);

                                // content_block_delta
                                let _ = tx.send(Ok(Event::default()
                                    .event("content_block_delta")
                                    .data(serde_json::json!({
                                        "type": "content_block_delta",
                                        "index": 0,
                                        "delta": {
                                            "type": "text_delta",
                                            "text": chunk.delta,
                                        }
                                    }).to_string())));
                            }

                            if chunk.finished {
                                // content_block_stop
                                let _ = tx.send(Ok(Event::default()
                                    .event("content_block_stop")
                                    .data("{\"type\":\"content_block_stop\"}".to_string())));

                                // message_delta
                                let _ = tx.send(Ok(Event::default()
                                    .event("message_delta")
                                    .data(serde_json::json!({
                                        "type": "message_delta",
                                        "delta": { "stop_reason": "end_turn" },
                                        "usage": { "output_tokens": text_accum.len() as i32 / 4 }
                                    }).to_string())));

                                // message_stop
                                let _ = tx.send(Ok(Event::default()
                                    .event("message_stop")
                                    .data("{\"type\":\"message_stop\"}".to_string())));
                            }
                        }
                    }
                    Err(e) => {
                        if let Some(kid) = used_key_id {
                            let mut s = state_clone.key_scheduler.write().await;
                            s.record_failure(&provider_slug, kid);
                        }
                        let _ = tx.send(Ok(Event::default()
                            .event("error")
                            .data(format!("{{\"type\":\"error\",\"error\":{{\"type\":\"api_error\",\"message\":\"{}\"}}}}", e))));
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(format!("{{\"type\":\"error\",\"error\":{{\"type\":\"api_error\",\"message\":\"{}\"}}}}", e))));
            }
        }

        if let Some(kid) = used_key_id {
            let mut scheduler = state_clone.key_scheduler.write().await;
            scheduler.record_success(&provider_slug, kid, start_time.elapsed().as_millis() as f64);
        }

        let latency_ms = start_time.elapsed().as_millis() as i32;
        log_api_call(&state_clone, &auth_clone, &provider_slug, &model, "chat", 0, 0, latency_ms).await;
    });

    let stream = BroadcastStream::new(rx).filter_map(|item| async {
        match item {
            Ok(Ok(event)) => Some(Ok::<_, std::convert::Infallible>(event)),
            _ => None,
        }
    });

    Ok(Sse::new(stream).into_response())
}

// ─── Anthropic response types ───────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct AnthropicResponse {
    id: String,
    #[allow(deprecated)]
    #[serde(rename = "type")]
    r#type: String,
    role: String,
    content: Vec<AnthropicContentBlock>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AnthropicContentBlock {
    Text {
        text: String,
        #[allow(deprecated)]
        #[serde(rename = "type")]
        r#type: String,
    },
}

#[derive(Debug, Serialize)]
struct AnthropicUsage {
    input_tokens: i32,
    output_tokens: i32,
}

// ─── Misc helpers ───────────────────────────────────────────────────────────

fn parse_model_string(model_str: &str) -> (String, String) {
    if let Some((provider, model)) = model_str.split_once('/') {
        (provider.to_string(), model.to_string())
    } else {
        (String::new(), model_str.to_string())
    }
}

fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get(SESSION_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn prepend_system(
    mut messages: Vec<InternalMessage>,
    _system_check: (),
) -> (Vec<InternalMessage>, Option<String>) {
    // In a real system we'd prepend the system prompt from the request.
    // For now we just return messages as-is.
    (messages, None)
}
