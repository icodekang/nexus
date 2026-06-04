//! Anthropic 兼容 API 接口模块 (`/v1/anthropic/*`)
//!
//! 请求/响应格式与 Anthropic Messages API 完全兼容
//!
//! 与 OpenAI 的主要区别：
//! - `max_tokens` 是必填项（Anthropic 总是生成到限制为止）
//! - System prompt 通过请求体的 `system` 字段传入
//! - 流式响应使用 SSE `event:` 行 (`message_start`, `content_block_delta`, …)
//! - 响应体格式为 `{"id":"...","content":[{"type":"text","text":"..."}]}`

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{sse::Event, IntoResponse, Response},
    Extension, Json,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use crate::state::AppState;
use models::Message as InternalMessage;
use provider_client::{ChatRequest as ProviderChatRequest, Message as ProviderMessage};

use super::shared::{
    add_rate_limit_headers, check_balance, create_client_from_source,
    default_session_id, extract_session_id, log_api_call, rate_limit_for_user,
    select_key_with_priority, validate_temperature,
};

// ─── 请求/响应类型 ───────────────────────────────────────────────

/// Anthropic 查询参数
#[derive(Debug, Deserialize)]
pub struct AnthropicQuery {
    /// 是否流式响应
    #[serde(default)]
    pub stream: Option<bool>,
}

/// Anthropic 请求体
#[derive(Debug, Deserialize)]
pub struct AnthropicRequest {
    /// 模型标识
    pub model: String,
    /// 消息列表
    pub messages: Vec<AnthropicMessage>,
    /// 最大生成 Token 数（Anthropic 必填）
    #[serde(default)]
    pub max_tokens: Option<i32>,
    /// 温度参数
    #[serde(rename = "temperature", default)]
    pub temperature: Option<f32>,
    /// 是否流式响应
    #[serde(rename = "stream", default)]
    pub stream: Option<bool>,
    /// System prompt
    #[serde(default)]
    pub system: Option<String>,
}

/// Anthropic 消息结构
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicMessage {
    /// 角色（user、assistant）
    pub role: String,
    /// 消息内容 — 支持字符串或 content block 数组
    ///   字符串:  "hello"
    ///   数组:    [{"type": "text", "text": "hello"}, {"type": "image", ...}]
    pub content: serde_json::Value,
}

/// 从 content 字段提取纯文本
fn extract_content_text(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(blocks) => blocks
            .iter()
            .filter_map(|b| {
                let is_text = b.get("type")
                    .and_then(|t| t.as_str())
                    .map(|t| t == "text")
                    .unwrap_or(false);
                if is_text {
                    b.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// Anthropic 响应体
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

/// Anthropic 内容块
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

/// Anthropic 使用量信息
#[derive(Debug, Serialize)]
struct AnthropicUsage {
    input_tokens: i32,
    output_tokens: i32,
}

// ─── 端点 ───────────────────────────────────────────────────────────────

/// POST /v1/anthropic/messages
///
/// Anthropic Messages API 兼容接口
///
/// # 功能
/// - 支持流式和非流式响应
/// - 支持 system prompt
/// - 与官方 Anthropic SDK 完全兼容
pub async fn anthropic_messages(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Query(_query): Query<AnthropicQuery>,
    Json(request): Json<AnthropicRequest>,
) -> Result<Response, ApiError> {
    let is_stream = request.stream.unwrap_or(false);

    // Transform Anthropic messages → internal format
    let internal_messages: Vec<InternalMessage> = request
        .messages
        .into_iter()
        .map(|m| InternalMessage {
            role: m.role,
            content: extract_content_text(&m.content),
            name: None,
        })
        .collect();

    // Inject system prompt into first user message if present
    let messages = prepend_system(internal_messages, request.system);

    let (provider_slug, model_slug) = parse_model_string(&request.model);
    let model = model_slug;
    let max_tokens = request.max_tokens; // required by Anthropic
    let sid = extract_session_id(&headers).or_else(|| default_session_id(&auth));

    if is_stream {
        anthropic_stream_handler(
            state,
            auth,
            model,
            messages,
            request.temperature.unwrap_or(1.0),
            max_tokens,
            sid,
            provider_slug,
        )
        .await
    } else {
        anthropic_blocking_handler(
            state,
            auth,
            model,
            messages,
            request.temperature.unwrap_or(1.0),
            max_tokens,
            sid,
            provider_slug,
        )
        .await
    }
}

/// GET /v1/anthropic/models
///
/// 获取可用模型列表（Anthropic 格式）
///
/// # 说明
/// 与官方 Anthropic Python SDK 的 `client.models.list()` 兼容
pub async fn anthropic_list_models(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use super::shared::list_models_impl;
    let models = list_models_impl(&state).await?;
    Ok(Json(serde_json::json!({ "models": models })))
}

// ─── Blocking handler ───────────────────────────────────────────────────────

async fn anthropic_blocking_handler(
    state: Arc<AppState>,
    auth: AuthContext,
    model: String,
    messages: Vec<InternalMessage>,
    temperature: f32,
    max_tokens: Option<i32>,
    session_id: Option<String>,
    provider_slug: String,
) -> Result<Response, ApiError> {
    let user_id = auth.user.id.to_string();
    let rpm = rate_limit_for_user();

    let (allowed, remaining, reset_time) = state
        .redis
        .check_rate_limit(&user_id, rpm, 60)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Rate limit check failed: {}", e)))?;

    if !allowed {
        return Err(ApiError::RateLimitExceeded);
    }
    check_balance(&state, auth.user.id).await?;

    validate_temperature(temperature)?;

    let db_model = state
        .db
        .get_model_by_id(&model)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?
        .ok_or_else(|| ApiError::ModelNotFound(model.clone()))?;

    if let Some(mt) = max_tokens {
        if mt <= 0 || mt > db_model.context_window {
            return Err(ApiError::InvalidRequest(format!(
                "max_tokens ({}) out of range",
                mt
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
        stream: false,
        extra: Default::default(),
    };

    let start_time = std::time::Instant::now();
    let (source, is_user_key) = select_key_with_priority(
        &state,
        auth.user.id,
        &provider_for_request,
        session_id.as_deref(),
    )
    .await?;
    let (client, key_id) = create_client_from_source(&provider_for_request, &source)?;

    let resp = client
        .chat(provider_request)
        .await
        .map_err(|e| ApiError::ProviderError(format!("Provider error: {}", e)))?;

    if !is_user_key {
        if let Some(kid) = key_id {
            let mut scheduler = state.key_scheduler.write().await;
            scheduler.record_success(&provider_for_request, kid, start_time.elapsed().as_millis() as f64);
        }
    }

    let latency_ms = start_time.elapsed().as_millis() as i32;
    let prompt_tokens = resp.usage.get("prompt_tokens").copied().unwrap_or(0);
    let completion_tokens = resp.usage.get("completion_tokens").copied().unwrap_or(0);

    log_api_call(
        &state,
        &auth,
        &provider_for_request,
        &model,
        "chat",
        prompt_tokens,
        completion_tokens,
        latency_ms,
    )
    .await;

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

// ─── 流式处理器 ───────────────────────────────────────────────────────

/// 流式响应处理器
async fn anthropic_stream_handler(
    state: Arc<AppState>,
    auth: AuthContext,
    model: String,
    messages: Vec<InternalMessage>,
    temperature: f32,
    max_tokens: Option<i32>,
    session_id: Option<String>,
    provider_slug: String,
) -> Result<Response, ApiError> {
    let (tx, rx) = broadcast::channel::<Result<Event, std::convert::Infallible>>(1024);
    let state_clone = state.clone();
    let auth_clone = auth.clone();

    let db_model = match state
        .db
        .get_model_by_id(&model)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?
    {
        Some(m) => m,
        None => return Err(ApiError::ModelNotFound(model.clone())),
    };

    let provider_for_request = if provider_slug.is_empty() {
        db_model.provider_id.clone()
    } else {
        provider_slug
    };

    let model_id = db_model.model_id.clone();

    tokio::spawn(async move {
        let mut used_key_id: Option<uuid::Uuid> = None;
        let is_user_key: bool;
        let estimated_input_tokens = messages
            .iter()
            .map(|m| (m.content.len() / 4) as i32)
            .sum::<i32>()
            .max(1);

        let (source, user_key) = match select_key_with_priority(
            &state_clone,
            auth_clone.user.id,
            &provider_for_request,
            session_id.as_deref(),
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("error")
                    .data(format!("{{\"error\": \"{}\"}}", e))));
                return;
            }
        };
        is_user_key = user_key;

        let client_result = match create_client_from_source(&provider_for_request, &source) {
            Ok((c, kid)) => {
                used_key_id = kid;
                Ok(c)
            }
            Err(e) => Err(e),
        };

        let start_time = std::time::Instant::now();

        match client_result {
            Ok(client) => {
                let provider_request = ProviderChatRequest {
                    provider: provider_for_request.clone(),
                    model: model_id.clone(),
                    messages: messages
                        .iter()
                        .map(|m| ProviderMessage {
                            role: m.role.clone(),
                            content: m.content.clone(),
                        })
                        .collect(),
                    temperature,
                    max_tokens,
                    stream: true,
                    extra: Default::default(),
                };

                let (stream_tx, mut stream_rx) = tokio::sync::mpsc::unbounded_channel::<provider_client::ChatChunk>();

                let tx_clone = tx.clone();
                let model_clone = model.clone();
                let forward_handle = tokio::spawn(async move {
                    let msg_id = format!("msg_{}", uuid::Uuid::new_v4());

                    let _ = tx_clone.send(Ok(Event::default().event("message_start").data(
                        serde_json::json!({
                            "type": "message_start",
                            "message": {
                                "id": msg_id,
                                "type": "message",
                                "role": "assistant",
                                "content": [],
                                "model": model_clone,
                            }
                        }).to_string(),
                    )));

                    let _ = tx_clone.send(Ok(Event::default().event("content_block_start").data(
                        serde_json::json!({
                            "type": "content_block_start",
                            "index": 0,
                            "content_block": { "type": "text", "text": "" }
                        }).to_string(),
                    )));

                    let mut text_accum = String::new();

                    while let Some(chunk) = stream_rx.recv().await {
                        if !chunk.delta.is_empty() {
                            text_accum.push_str(&chunk.delta);

                            let _ = tx_clone.send(Ok(Event::default()
                                .event("content_block_delta")
                                .data(
                                    serde_json::json!({
                                        "type": "content_block_delta",
                                        "index": 0,
                                        "delta": { "type": "text_delta", "text": chunk.delta }
                                    })
                                    .to_string(),
                                )));
                        }

                        if chunk.finished {
                            // content_block_stop
                            let _ = tx_clone.send(Ok(Event::default()
                                .event("content_block_stop")
                                .data(serde_json::json!({ "type": "content_block_stop", "index": 0 }).to_string())));
                            // message_delta (usage/stats)
                            let _ = tx_clone.send(Ok(Event::default()
                                .event("message_delta")
                                .data(serde_json::json!({
                                    "type": "message_delta",
                                    "delta": {
                                        "stop_reason": chunk.finish_reason.as_deref().unwrap_or("end_turn"),
                                        "stop_sequence": null,
                                    },
                                    "usage": { "output_tokens": text_accum.len() / 4 }
                                }).to_string())));
                            // message_stop
                            let _ = tx_clone.send(Ok(Event::default()
                                .event("message_stop")
                                .data(serde_json::json!({ "type": "message_stop" }).to_string())));
                            break;
                        }
                    }
                });

                match client.chat_stream(provider_request, stream_tx).await {
                    Ok(()) => { let _ = forward_handle.await; }
                    Err(e) => {
                        if !is_user_key {
                            if let Some(kid) = used_key_id {
                                let mut scheduler = state_clone.key_scheduler.write().await;
                                scheduler.record_failure(&provider_for_request, kid);
                            }
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
                    .data(format!("{{\"error\": \"{}\"}}", e))));
            }
        }

        if !is_user_key {
            if let Some(kid) = used_key_id {
                let mut scheduler = state_clone.key_scheduler.write().await;
                scheduler.record_success(&provider_for_request, kid, start_time.elapsed().as_millis() as f64);
            }
        }

        let latency_ms = start_time.elapsed().as_millis() as i32;
        log_api_call(
            &state_clone,
            &auth_clone,
            &provider_for_request,
            &model,
            "chat",
            estimated_input_tokens,
            0,
            latency_ms,
        )
        .await;
    });

    let stream = BroadcastStream::new(rx).filter_map(|item| async {
        match item {
            Ok(Ok(event)) => Some(Ok::<_, std::convert::Infallible>(event)),
            _ => None,
        }
    });

    Ok(IntoResponse::into_response(axum::response::Sse::new(
        stream,
    )))
}

// ─── 辅助函数 ───────────────────────────────────────────────────────────

/// 预处理 System Prompt
///
/// 如果提供了 system 参数，将其添加到消息列表的开头
fn prepend_system(messages: Vec<InternalMessage>, system: Option<String>) -> Vec<InternalMessage> {
    match system {
        Some(sys) if !sys.is_empty() => {
            let mut result = vec![InternalMessage {
                role: "system".to_string(),
                content: sys,
                name: None,
            }];
            result.extend(messages);
            result
        }
        _ => messages,
    }
}

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
