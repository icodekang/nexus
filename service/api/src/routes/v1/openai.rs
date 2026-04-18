//! OpenAI 兼容 API 接口模块 (`/v1/openai/*`)
//!
//! 请求/响应格式与 OpenAI Chat Completions API 完全兼容

use std::sync::Arc;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response, sse::Event},
    Json, Extension,
};
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use futures_util::StreamExt;
use serde::Deserialize;

use crate::error::ApiError;
use crate::state::AppState;
use crate::middleware::auth::AuthContext;
use models::{Message as InternalMessage, ChatChunk};
use provider_client::{
    ChatRequest as ProviderChatRequest,
    Message as ProviderMessage,
    HttpProviderClient,
};

use super::shared::{
    rate_limit_for_plan, select_key, create_client, record_result,
    extract_session_id, default_session_id,
    check_subscription, check_token_quota,
    validate_temperature, add_rate_limit_headers, log_api_call,
};

// ─── 请求类型 ───────────────────────────────────────────────────────────

/// OpenAI 查询参数
#[derive(Debug, Deserialize)]
pub struct OpenAIQuery {
    /// 是否流式响应
    pub stream: Option<bool>,
}

/// OpenAI 客户端请求体
#[derive(Debug, Deserialize)]
pub struct OpenAIClientRequest {
    /// 模型标识
    pub model: String,
    /// 消息列表
    pub messages: Vec<OpenAIMessage>,
    /// 温度参数
    #[serde(default)]
    pub temperature: Option<f32>,
    /// 最大生成 Token 数
    #[serde(default)]
    pub max_tokens: Option<i32>,
    /// 是否流式响应
    #[serde(rename = "stream", default)]
    pub stream: Option<bool>,
    /// Top-p 采样参数
    #[serde(default)]
    pub top_p: Option<f32>,
    /// 停止序列
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    /// 用户标识（可选）
    #[serde(default)]
    pub user: Option<String>,
}

/// OpenAI 消息结构
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIMessage {
    /// 角色（system、user、assistant）
    pub role: String,
    /// 消息内容
    pub content: String,
    /// 名称（可选，用于 user 消息）
    #[serde(default)]
    pub name: Option<String>,
}

impl From<OpenAIMessage> for InternalMessage {
    fn from(m: OpenAIMessage) -> Self {
        InternalMessage { role: m.role, content: m.content, name: m.name }
    }
}

// ─── 端点 ───────────────────────────────────────────────────────────────

/// POST /v1/openai/chat/completions
///
/// OpenAI Chat Completions API 兼容接口
///
/// # 说明
/// 与官方 OpenAI Python SDK 的 `client.chat.completions.create()` 兼容
pub async fn openai_chat_completions(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Query(_query): Query<OpenAIQuery>,
    Json(request): Json<OpenAIClientRequest>,
) -> Result<Response, ApiError> {
    let is_stream = request.stream.unwrap_or(false);
    let (provider_slug, model_slug) = parse_model_string(&request.model);
    let session_id = extract_session_id(&headers).or_else(|| default_session_id(&auth));

    unified_chat(
        state, auth, provider_slug, model_slug,
        request.messages.into_iter().map(Into::into).collect(),
        request.temperature.unwrap_or(0.7),
        request.max_tokens,
        is_stream,
        session_id,
    )
    .await
}

/// GET /v1/openai/models
///
/// 获取可用模型列表（OpenAI 格式）
///
/// # 说明
/// 与官方 OpenAI Python SDK 的 `client.models.list()` 兼容
pub async fn openai_list_models(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use super::shared::list_models_impl;
    let models = list_models_impl(&state).await?;
    Ok(Json(serde_json::json!({ "object": "list", "data": models })))
}

// ─── 统一聊天处理器 ────────────────────────────────────────────────────

/// 统一的聊天处理函数
async fn unified_chat(
    state: Arc<AppState>,
    auth: AuthContext,
    provider_slug: String,
    model: String,
    messages: Vec<InternalMessage>,
    temperature: f32,
    max_tokens: Option<i32>,
    is_stream: bool,
    _session_id: Option<String>,
) -> Result<Response, ApiError> {
    let user_id = auth.user.id.to_string();
    let rpm = rate_limit_for_plan(&auth.user.subscription_plan);

    // Rate limit
    let (allowed, remaining, reset_time) = state
        .redis
        .check_rate_limit(&user_id, rpm, 60)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Rate limit check failed: {}", e)))?;

    if !allowed {
        return Err(ApiError::RateLimitExceeded);
    }

    check_subscription(&auth.user)?;
    check_token_quota(&state, &auth.user).await?;

    // Look up model
    let db_model = state
        .db
        .get_model_by_slug(&model)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?
        .ok_or_else(|| ApiError::ModelNotFound(model.clone()))?;

    validate_temperature(temperature)?;

    if let Some(mt) = max_tokens {
        if mt <= 0 || mt > db_model.context_window {
            return Err(ApiError::InvalidRequest(format!(
                "max_tokens ({}) out of range", mt
            )));
        }
    }

    let provider_for_request = if provider_slug.is_empty() {
        db_model.provider_id.clone()
    } else {
        provider_slug.clone()
    };

    let provider_messages: Vec<ProviderMessage> = messages
        .iter()
        .map(|m| ProviderMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    let provider_request = ProviderChatRequest {
        provider: provider_for_request.clone(),
        model: db_model.model_id.clone(),
        messages: provider_messages,
        temperature,
        max_tokens,
        stream: is_stream,
        extra: Default::default(),
    };

    let start_time = std::time::Instant::now();

    if is_stream {
        openai_stream_handler(state, auth, provider_slug, model, start_time,
                              rpm, remaining, reset_time,
                              provider_for_request, provider_request)
        .await
    } else {
        openai_blocking_handler(state, auth, provider_slug, model, start_time,
                               rpm, remaining, reset_time,
                               provider_for_request, provider_request)
        .await
    }
}

// ─── 非流式处理器 ──────────────────────────────────────────────────

/// 非流式响应处理器
async fn openai_blocking_handler(
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

    // Primary provider with session affinity
    let selected_key = select_key(
        &state,
        &provider_for_request,
        default_session_id(&auth).as_deref(),
    )
    .await?;
    let (client, key_id) = create_client(&provider_for_request, selected_key)?;

    provider_request.provider = provider_for_request.clone();
    match client.chat(provider_request.clone()).await {
        Ok(resp) => {
            provider_resp = Some(resp);
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

    log_api_call(&state, &auth, &provider_for_request, &model_name,
                 "chat", prompt_tokens, completion_tokens, latency_ms).await;

    let mut response = Json(provider_resp).into_response();
    add_rate_limit_headers(&mut response, rpm, remaining, reset_time);
    Ok(response)
}

// ─── 流式处理器 ───────────────────────────────────────────────────────

/// 流式响应处理器
async fn openai_stream_handler(
    state: Arc<AppState>,
    auth: AuthContext,
    _provider_slug: String,
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
    let sid = default_session_id(&auth);

    tokio::spawn(async move {
        let mut used_key_id: Option<uuid::Uuid> = None;

        let selected_key = {
            let mut scheduler = state_clone.key_scheduler.write().await;
            scheduler.tick();
            sid.as_ref()
                .and_then(|s| scheduler.select_key_for_session(&provider_clone, s))
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
                            if chunk.finished {
                                break;
                            }
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

    let mut response = IntoResponse::into_response(axum::response::Sse::new(stream));
    add_rate_limit_headers(&mut response, rpm, remaining, reset_time);
    Ok(response)
}

// ─── 辅助函数 ───────────────────────────────────────────────────────────

/// 解析模型字符串
    ///
    /// 支持 "provider/model" 或 "model" 格式
fn parse_model_string(model_str: &str) -> (String, String) {
    if let Some((provider, model)) = model_str.split_once('/') {
        (provider.to_string(), model.to_string())
    } else {
        (String::new(), model_str.to_string())
    }
}
