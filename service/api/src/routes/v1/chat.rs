//! 统一 Chat 接口路由模块
//! 提供 /v1/chat/completions、/v1/completions、/v1/embeddings 接口
//!
//! 特性：
//! - 支持流式和非流式响应
//! - 会话亲和性（同一会话使用相同 API Key）
//! - 多 Provider 故障转移
//! - ZeroToken 用户使用浏览器模拟器

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json, Extension,
};
use std::sync::Arc;
use futures_util::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use serde::Deserialize;

use crate::error::ApiError;
use crate::state::AppState;
use crate::middleware::auth::AuthContext;
use models::{ChatRequest, ChatResponse, Message, Usage, User, SubscriptionPlan};
use provider_client::{
    ChatRequest as ProviderChatRequest,
    Message as ProviderMessage,
    ProviderClient, ProviderClientFactory, HttpProviderClient,
    BrowserEmulatorFactory,
};
use router::key_scheduler::SelectedKey;

/// 会话亲和性 Header
const SESSION_HEADER: &str = "x-session-id";

// 速率限制常量（请求/分钟）
const FREE_RPM: i64 = 10;
const ZEROTOKEN_RPM: i64 = 30;
const MONTHLY_RPM: i64 = 60;
const YEARLY_RPM: i64 = 120;
const TEAM_RPM: i64 = 300;
const ENTERPRISE_RPM: i64 = 1000;

/// 根据套餐获取速率限制
fn rate_limit_for_plan(plan: &SubscriptionPlan) -> i64 {
    match plan {
        SubscriptionPlan::None => FREE_RPM,
        SubscriptionPlan::ZeroToken => ZEROTOKEN_RPM,
        SubscriptionPlan::Monthly => MONTHLY_RPM,
        SubscriptionPlan::Yearly => YEARLY_RPM,
        SubscriptionPlan::Team => TEAM_RPM,
        SubscriptionPlan::Enterprise => ENTERPRISE_RPM,
    }
}

/// 选择 Provider 的 API Key（支持会话亲和性）
    ///
    /// 如果提供 session_id，调度器会尽量为同一会话选择相同的 Key
    /// 这样可以提高用户体验（会话一致性）
async fn select_provider_key(
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

/// 从选中的 Key 创建 Provider 客户端
    ///
    /// 同时返回 key_id 以便调用者记录成功/失败
fn create_client_with_key(
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
            // Fallback to env-based key (legacy mode — no load balancing)
            let client = HttpProviderClient::new(provider_slug)
                .map_err(|e| ApiError::ProviderError(format!("Failed to create client: {}", e)))?;
            Ok((Arc::new(client), None))
        }
    }
}

/// 为 ZeroToken 用户创建浏览器模拟器客户端
    ///
    /// ZeroToken 用户使用浏览器模拟器而不是直接调用 API
fn create_zero_token_client(
    provider: &str,
) -> Result<(Arc<dyn ProviderClient>, Option<uuid::Uuid>), ApiError> {
    let client = BrowserEmulatorFactory::create(provider)
        .map_err(|e| ApiError::ProviderError(format!("Failed to create browser emulator: {}", e)))?;
    Ok((client, None)) // Browser emulator doesn't use API keys
}

/// 从账户池获取浏览器模拟器客户端
    ///
    /// 优先从池中获取可用账户，如果池为空则创建新客户端
async fn get_zero_token_client_from_pool(
    state: &Arc<AppState>,
    provider: &str,
) -> Result<(Arc<dyn ProviderClient>, Option<uuid::Uuid>), ApiError> {
    // Try to get an available account from the pool
    if let Some(client) = state.account_pool.get_client(provider).await {
        return Ok((client, None));
    }

    // Fallback to creating a new client if no accounts available
    tracing::warn!("No authenticated accounts in pool for provider {}, creating new client", provider);
    create_zero_token_client(provider)
}

/// 判断用户是否使用 ZeroToken（浏览器模拟器）访问
fn is_zero_token_user(subscription_plan: SubscriptionPlan) -> bool {
    subscription_plan.is_zero_token()
}

/// 根据模型 Provider 获取对应的浏览器模拟器 Provider
fn get_zero_token_provider(model_provider: &str) -> &'static str {
    // Map model providers to browser emulator providers
    match model_provider {
        "openai" | "chatgpt" => "chatgpt",
        "anthropic" | "claude" => "claude",
        // Default to claude for unknown providers
        _ => "claude",
    }
}

/// 记录 Key 请求结果
    ///
    /// 更新 Key 的压力值、失败计数和会话绑定状态
async fn record_key_result(
    state: &Arc<AppState>,
    provider_slug: &str,
    key_id: Option<uuid::Uuid>,
    latency_ms: i32,
    success: bool,
) {
    if let Some(key_id) = key_id {
        let mut scheduler = state.key_scheduler.write().await;
        if success {
            scheduler.record_success(provider_slug, key_id, latency_ms as f64);
        } else {
            scheduler.record_failure(provider_slug, key_id);
        }
    }
}

/// Chat 查询参数
#[derive(Debug, Deserialize)]
pub struct ChatQuery {
    /// 是否流式响应
    pub stream: Option<bool>,
}

/// 从 Header 中提取会话 ID
    ///
    /// 优先使用 `x-session-id` Header，
    /// 其次使用 user_id（对于没有浏览器会话的 API Key 调用者）
fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get(SESSION_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// POST /v1/chat/completions
///
/// 统一的聊天补全接口
///
/// # 功能
/// - 支持流式和非流式响应
/// - 会话亲和性（通过 x-session-id Header）
/// - 多 Provider 故障转移
/// - ZeroToken 用户使用浏览器模拟器
///
/// # 请求参数
/// - model: 模型标识（如 anthropic/claude-3-opus 或 claude-3-opus）
/// - messages: 消息列表
/// - temperature: 温度参数（0.0-2.0）
/// - max_tokens: 最大生成 Token 数
/// - stream: 是否流式响应
///
/// # 认证
/// 需要有效的 API Key 或 JWT Token
pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ChatQuery>,
    headers: HeaderMap,
    Json(request): Json<ChatRequest>,
) -> Result<Response, ApiError> {
    let is_stream = query.stream.unwrap_or(request.stream);

    // 1. Rate limiting
    let user_id = auth.user.id.to_string();
    let rpm = rate_limit_for_plan(&auth.user.subscription_plan);
    let (allowed, remaining, reset_time) = state.redis
        .check_rate_limit(&user_id, rpm, 60)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Rate limit check failed: {}", e)))?;

    if !allowed {
        return Err(ApiError::RateLimitExceeded);
    }

    // Extract session ID for key affinity.
    // Priority: x-session-id header > user_id (for API-key callers without browser session).
    // This ensures every authenticated user gets stable key binding even without frontend session tracking.
    let session_id = extract_session_id(&headers)
        .or_else(|| Some(auth.user.id.to_string()));

    // 2. Parse model string to get provider and model
    let (provider_slug, model_slug) = parse_model_string(&request.model);

    // 3. Look up the model
    let model = state.db.get_model_by_slug(&model_slug)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?
        .ok_or_else(|| ApiError::ModelNotFound(request.model.clone()))?;

    // 4. Check subscription status
    check_subscription(&auth.user)?;

    // 5. Check token quota for this billing period
    check_token_quota(&state, &auth.user).await?;

    // 6. Validate request parameters
    validate_chat_request(&request, model.context_window)?;

    // 5. Check if model supports streaming if requested
    if is_stream && !model.capabilities.iter().any(|c| c == "streaming" || c == "stream") {
        return Err(ApiError::InvalidRequest(
            "Model does not support streaming".to_string()
        ));
    }

    let provider_for_request = if provider_slug.is_empty() {
        model.provider_id.clone()
    } else {
        provider_slug
    };

    // 6. Build provider list for fallback (primary + alternates)
    let mut providers_to_try: Vec<String> = vec![provider_for_request.clone()];
    if let Ok(all_providers) = state.db.list_providers().await {
        for p in &all_providers {
            if p.is_active && p.slug != provider_for_request {
                providers_to_try.push(p.slug.clone());
            }
        }
    }

    // 7. Check if user is ZeroToken subscriber (browser-based access)
    let is_zero_token = is_zero_token_user(auth.user.subscription_plan);
    let zero_token_provider = if is_zero_token {
        Some(get_zero_token_provider(&provider_for_request))
    } else {
        None
    };

    // Build request for provider client
    let provider_messages: Vec<ProviderMessage> = request.messages.iter().map(|m| {
        ProviderMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        }
    }).collect();

    let provider_request = ProviderChatRequest {
        provider: provider_for_request.clone(),
        model: model.model_id.clone(),
        messages: provider_messages,
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        stream: is_stream,
        extra: std::collections::HashMap::new(),
    };

    let start_time = std::time::Instant::now();

    if is_stream {
        // =============================================================
        // STREAMING PATH — now uses session-aware key scheduling
        // =============================================================
        let (tx, rx) = broadcast::channel::<Result<Event, std::convert::Infallible>>(1024);

        let model_name = request.model.clone();
        let state_clone = state.clone();
        let auth_clone = auth.clone();
        let provider_clone = provider_for_request.clone();
        let model_slug_clone = model.slug.clone();
        let session_id_clone = session_id.clone();
        let is_zero_token_stream = is_zero_token;
        let zero_token_provider_stream = zero_token_provider;

        tokio::spawn(async move {
            let mut total_output_tokens: i32 = 0;
            let mut used_key_id: Option<uuid::Uuid> = None;

            // ZeroToken users use browser emulator instead of API keys
            let client_result = if is_zero_token_stream {
                if let Some(zt_provider) = zero_token_provider_stream {
                    get_zero_token_client_from_pool(&state_clone, zt_provider).await
                } else {
                    // Fallback to HTTP client if no zero-token provider
                    HttpProviderClient::new(&provider_clone)
                        .map(|c| (Arc::new(c) as Arc<dyn ProviderClient>, None))
                        .map_err(|e| ApiError::Internal(anyhow::anyhow!("HTTP client error: {}", e)))
                }
            } else {
                // Select key with session affinity
                let selected_key = {
                    let mut scheduler = state_clone.key_scheduler.write().await;
                    scheduler.tick();
                    session_id_clone.as_ref().map(|sid| {
                        scheduler.select_key_for_session(&provider_clone, sid)
                    }).flatten()
                };

                match &selected_key {
                    Some(sk) => {
                        used_key_id = Some(sk.key.id);
                        HttpProviderClient::new_with_key(&provider_clone, &sk.key)
                            .map(|c| (Arc::new(c) as Arc<dyn ProviderClient>, Some(sk.key.id)))
                            .map_err(|e| ApiError::Internal(anyhow::anyhow!("HTTP client error: {}", e)))
                    }
                    None => HttpProviderClient::new(&provider_clone)
                        .map(|c| (Arc::new(c) as Arc<dyn ProviderClient>, None))
                        .map_err(|e| ApiError::Internal(anyhow::anyhow!("HTTP client error: {}", e))),
                }
            };

            match client_result {
                Ok((client, _)) => {
                    match client.chat_stream(provider_request).await {
                        Ok(chunks) => {
                            for chunk in chunks {
                                if !chunk.delta.is_empty() {
                                    total_output_tokens += 1;
                                }

                                let event = if chunk.finished {
                                    let chat_chunk = models::ChatChunk::new(&model_name, "", true);
                                    Event::default()
                                        .event("message")
                                        .data(serde_json::to_string(&chat_chunk).unwrap_or_default())
                                } else {
                                    let chat_chunk = models::ChatChunk::new(&model_name, &chunk.delta, false);
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
                            // Record failure and evict session binding (only for API key users)
                            if !is_zero_token_stream {
                                if let Some(kid) = used_key_id {
                                    let mut scheduler = state_clone.key_scheduler.write().await;
                                    scheduler.record_failure(&provider_clone, kid);
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
                        .data(format!("{{\"error\": \"Failed to create provider client: {}\"}}", e))));
                }
            }

            // Record success for the key (only for API key users, not ZeroToken)
            if !is_zero_token_stream {
                if let Some(kid) = used_key_id {
                    let latency_ms = start_time.elapsed().as_millis() as i32;
                    let mut scheduler = state_clone.key_scheduler.write().await;
                    scheduler.record_success(&provider_clone, kid, latency_ms as f64);
                }
            }

            // Log streaming API call
            let latency_ms = start_time.elapsed().as_millis() as i32;
            log_api_call(
                &state_clone,
                &auth_clone,
                &provider_clone,
                &model_slug_clone,
                "chat",
                0,
                total_output_tokens,
                latency_ms,
            ).await;
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
    } else {
        // =============================================================
        // NON-STREAMING PATH — session-aware key scheduling
        // =============================================================
        let mut last_error = None;
        let mut provider_resp = None;
        let mut used_provider = provider_for_request.clone();

        // ZeroToken users use browser emulator instead of API keys
        let (client, key_id) = if let Some(zt_provider) = zero_token_provider {
            used_provider = zt_provider.to_string();
            get_zero_token_client_from_pool(&state, zt_provider).await?
        } else {
            // Try primary provider with session-aware key selection
            let selected_key = select_provider_key(&state, &provider_for_request, session_id.as_deref()).await?;
            create_client_with_key(&provider_for_request, selected_key)?
        };

        // client is Arc<dyn ProviderClient> (already unwrapped from Result)
        let mut req = provider_request.clone();
        req.provider = used_provider.clone();
        match client.chat(req).await {
            Ok(resp) => {
                provider_resp = Some(resp);
                let latency_ms = start_time.elapsed().as_millis() as i32;
                // Only record key result for non-ZeroToken (API key) requests
                if !is_zero_token {
                    record_key_result(&state, &provider_for_request, key_id, latency_ms, true).await;
                }
            }
            Err(e) => {
                tracing::warn!("Primary provider {} failed: {}", provider_for_request, e);
                last_error = Some(e);
                // Only record key result for non-ZeroToken (API key) requests
                if !is_zero_token {
                    record_key_result(&state, &provider_for_request, key_id, 0, false).await;
                }
            }
        }

        // Fallback to other providers if primary failed
        // ZeroToken users fallback to other browser emulator providers
        if provider_resp.is_none() && !is_zero_token {
            for try_provider in &providers_to_try {
                if *try_provider == provider_for_request {
                    continue; // Already tried
                }

                // Fallback providers don't have session context — use pressure-only selection
                let selected_key = select_provider_key(&state, try_provider, None).await?;
                let (client, key_id) = create_client_with_key(try_provider, selected_key)?;

                let mut req = provider_request.clone();
                req.provider = try_provider.clone();
                match client.chat(req).await {
                    Ok(resp) => {
                        provider_resp = Some(resp);
                        used_provider = try_provider.clone();
                        let latency_ms = start_time.elapsed().as_millis() as i32;
                        record_key_result(&state, try_provider, key_id, latency_ms, true).await;
                        break;
                    }
                    Err(e) => {
                        tracing::warn!("Fallback provider {} failed: {}", try_provider, e);
                        last_error = Some(e);
                        record_key_result(&state, try_provider, key_id, 0, false).await;
                    }
                }
            }
        } else if provider_resp.is_none() && is_zero_token {
            // ZeroToken fallback: try other browser emulator providers
            let fallback_providers = vec!["claude", "chatgpt"];
            for zt_provider in fallback_providers {
                if zt_provider == zero_token_provider.unwrap_or("") {
                    continue;
                }

                let (client, _key_id) = get_zero_token_client_from_pool(&state, zt_provider).await?;
                let mut req = provider_request.clone();
                req.provider = zt_provider.to_string();
                match client.chat(req).await {
                    Ok(resp) => {
                        provider_resp = Some(resp);
                        used_provider = zt_provider.to_string();
                        break;
                    }
                    Err(e) => {
                        tracing::warn!("ZeroToken fallback provider {} failed: {}", zt_provider, e);
                        last_error = Some(e);
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

        let response = ChatResponse {
            id: provider_resp.id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: request.model.clone(),
            choices: vec![models::Choice {
                index: 0,
                message: Message {
                    role: provider_resp.message.role,
                    content: provider_resp.message.content,
                    name: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            },
        };

        log_api_call(
            &state,
            &auth,
            &used_provider,
            &model.slug,
            "chat",
            prompt_tokens,
            completion_tokens,
            latency_ms,
        ).await;

        let mut http_response = Json(response).into_response();
        add_rate_limit_headers(&mut http_response, rpm, remaining, reset_time);
        Ok(http_response)
    }
}

/// 添加速率限制 Header（OpenRouter 兼容）
fn add_rate_limit_headers(response: &mut Response, limit: i64, remaining: i64, reset: i64) {
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", limit.to_string().parse().unwrap());
    headers.insert("X-RateLimit-Remaining", remaining.to_string().parse().unwrap());
    headers.insert("X-RateLimit-Reset", reset.to_string().parse().unwrap());
}

/// 检查用户订阅是否有效
fn check_subscription(user: &User) -> Result<(), ApiError> {
    match user.subscription_plan {
        SubscriptionPlan::None => {
            // Free tier is always "active" but has limited quota
        }
        SubscriptionPlan::ZeroToken | SubscriptionPlan::Monthly | SubscriptionPlan::Yearly | SubscriptionPlan::Team | SubscriptionPlan::Enterprise => {
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

/// 检查用户在当前计费周期的 Token 配额
async fn check_token_quota(state: &AppState, user: &User) -> Result<(), ApiError> {
    let quota = user.subscription_plan.monthly_token_quota();
    if quota == i64::MAX {
        return Ok(()); // Enterprise: unlimited
    }

    let now = chrono::Utc::now();
    let period_start = user.subscription_start.unwrap_or(now);
    let period_end = user.subscription_end.unwrap_or(
        now + chrono::Duration::days(user.subscription_plan.billing_cycle_days())
    );

    if now > period_end {
        return Err(ApiError::SubscriptionExpired);
    }

    let used = state.db
        .get_user_token_usage_in_period(user.id, period_start, period_end)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to check token usage: {}", e)))?;

    if used >= quota {
        return Err(ApiError::InvalidRequest(
            format!("Token quota exceeded. Used {} / {} tokens this period. Upgrade your plan for more tokens.", used, quota)
        ));
    }

    Ok(())
}

/// 记录 API 调用到数据库
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

/// POST /v1/completions
///
/// 文本补全接口（未实现）
///
/// # 说明
/// 请使用 /v1/chat/completions 接口
pub async fn completions(
    State(_state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Json(_request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ApiError> {
    return Err::<axum::response::Response, _>(ApiError::NotImplemented(
        "Completions endpoint not implemented. Use /v1/chat/completions".to_string()
    ))
}

/// POST /v1/embeddings
///
/// 向量嵌入接口
///
/// # 功能
/// 将文本转换为向量表示
///
/// # 参数
/// - model: 模型名称
/// - input: 要嵌入的文本或文本列表
pub async fn embeddings(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<models::EmbeddingsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    check_subscription(&auth.user)?;

    let user_id = auth.user.id.to_string();
    let rpm = rate_limit_for_plan(&auth.user.subscription_plan);
    let (allowed, remaining, reset_time) = state.redis
        .check_rate_limit(&user_id, rpm, 60)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Rate limit check failed: {}", e)))?;

    if !allowed {
        return Err(ApiError::RateLimitExceeded);
    }

    let provider_id = &request.model;

    let provider_client = ProviderClientFactory::create(provider_id)
        .map_err(|e| ApiError::ProviderError(e.to_string()))?;

    let start_time = std::time::Instant::now();

    let provider_request = provider_client::EmbeddingsRequest {
        model: request.model.clone(),
        inputs: request.input.clone(),
    };

    let provider_resp = provider_client
        .embeddings(provider_request)
        .await
        .map_err(|e| ApiError::ProviderError(format!("Provider error: {}", e)))?;

    let latency_ms = start_time.elapsed().as_millis() as i32;
    let total_tokens = provider_resp.embeddings.iter().map(|e| e.len() as i32).sum::<i32>();

    log_api_call(
        &state,
        &auth,
        provider_id,
        provider_id,
        "embedding",
        total_tokens,
        0,
        latency_ms,
    ).await;

    let response = models::EmbeddingsResponse::new(
        &request.model,
        provider_resp.embeddings,
    );

    let mut http_response = Json(response).into_response();
    add_rate_limit_headers(&mut http_response, rpm, remaining, reset_time);
    Ok(http_response)
}

/// 解析模型字符串
    ///
    /// 支持两种格式：
    /// - "provider/model" (如 "anthropic/claude-3-opus")
    /// - "model" (如 "claude-3-opus")
fn parse_model_string(model_str: &str) -> (String, String) {
    if let Some((provider, model)) = model_str.split_once('/') {
        (provider.to_string(), model.to_string())
    } else {
        (String::new(), model_str.to_string())
    }
}

/// 验证 Chat 请求参数
    ///
    /// 检查以下参数：
    /// - temperature: 必须在 0.0-2.0 之间
    /// - max_tokens: 必须大于 0 且不超过上下文窗口
    /// - top_p: 必须在 0.0-1.0 之间
    /// - messages: 不能为空
    /// - message roles: 必须是 system、user、assistant 或 tool
fn validate_chat_request(request: &ChatRequest, context_window: i32) -> Result<(), ApiError> {
    if request.temperature < 0.0 || request.temperature > 2.0 {
        return Err(ApiError::InvalidRequest(
            "temperature must be between 0.0 and 2.0".to_string()
        ));
    }

    if let Some(max_tokens) = request.max_tokens {
        if max_tokens <= 0 {
            return Err(ApiError::InvalidRequest(
                "max_tokens must be greater than 0".to_string()
            ));
        }
        if max_tokens > context_window {
            return Err(ApiError::InvalidRequest(
                format!("max_tokens ({}) exceeds model context window ({})", max_tokens, context_window)
            ));
        }
    }

    if let Some(top_p) = request.top_p {
        if top_p < 0.0 || top_p > 1.0 {
            return Err(ApiError::InvalidRequest(
                "top_p must be between 0.0 and 1.0".to_string()
            ));
        }
    }

    if request.messages.is_empty() {
        return Err(ApiError::InvalidRequest(
            "messages must not be empty".to_string()
        ));
    }

    for msg in &request.messages {
        match msg.role.as_str() {
            "system" | "user" | "assistant" | "tool" => {}
            _ => return Err(ApiError::InvalidRequest(
                format!("invalid message role: '{}'. Must be one of: system, user, assistant, tool", msg.role)
            )),
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// 批量查询（多模型对比）
// ═══════════════════════════════════════════════════════════════════════════════

use models::{BatchChatRequest, BatchChatResponse, ModelResult};

/// POST /v1/chat/batch — 多模型并行查询 + LLM-as-Judge 评分
pub async fn chat_batch(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    headers: HeaderMap,
    Json(request): Json<BatchChatRequest>,
) -> Result<Json<BatchChatResponse>, ApiError> {
    let total_start = std::time::Instant::now();

    // 1. Rate limiting
    let user_id = auth.user.id.to_string();
    let rpm = rate_limit_for_plan(&auth.user.subscription_plan);
    let (allowed, _, _) = state.redis
        .check_rate_limit(&user_id, rpm, 60)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Rate limit check failed: {}", e)))?;
    if !allowed {
        return Err(ApiError::RateLimitExceeded);
    }

    // 2. Validate
    if request.messages.is_empty() {
        return Err(ApiError::InvalidRequest("messages must not be empty".to_string()));
    }

    // 3. Check subscription
    check_subscription(&auth.user)?;
    check_token_quota(&state, &auth.user).await?;

    // 4. Get the user's last question
    let user_query = request.messages.iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    // 5. Select models
    let all_models = state.db.list_models().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list models: {}", e)))?;

    let selected_model_slugs = if let Some(ref models) = request.models {
        models.clone()
    } else {
        smart_select_models(&user_query, &all_models)
    };

    if selected_model_slugs.is_empty() {
        return Err(ApiError::InvalidRequest("No models available".to_string()));
    }

    // 6. Query all models in parallel
    let session_id = extract_session_id(&headers)
        .or_else(|| Some(auth.user.id.to_string()));

    let mut tasks = Vec::new();

    for model_slug in &selected_model_slugs {
        let state_clone = state.clone();
        let slug = model_slug.clone();
        let messages = request.messages.clone();
        let max_tokens = request.max_tokens;
        let sid = session_id.clone();

        // Find model info
        let model_info = all_models.iter().find(|m| m.slug == *slug).cloned();

        let task = tokio::spawn(async move {
            let start = std::time::Instant::now();

            let model = match model_info {
                Some(m) => m,
                None => return ModelResult {
                    model: slug.clone(),
                    provider: String::new(),
                    content: String::new(),
                    score: 0.0,
                    reason: String::new(),
                    latency_ms: 0,
                    success: false,
                    error: Some("Model not found".to_string()),
                    usage: Usage::new(0, 0),
                },
            };

            let provider_slug = model.provider_id.clone();

            // Select key
            let selected_key = {
                let mut scheduler = state_clone.key_scheduler.write().await;
                scheduler.tick();
                sid.as_ref().and_then(|s| scheduler.select_key_for_session(&provider_slug, s))
            };

            // Create client
            let client_result = match &selected_key {
                Some(sk) => HttpProviderClient::new_with_key(&provider_slug, &sk.key)
                    .map(|c| (Arc::new(c) as Arc<dyn ProviderClient>, Some(sk.key.id)))
                    .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e))),
                None => HttpProviderClient::new(&provider_slug)
                    .map(|c| (Arc::new(c) as Arc<dyn ProviderClient>, None))
                    .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e))),
            };

            let (client, key_id) = match client_result {
                Ok(v) => v,
                Err(e) => return ModelResult {
                    model: slug.clone(),
                    provider: provider_slug,
                    content: String::new(),
                    score: 0.0,
                    reason: String::new(),
                    latency_ms: start.elapsed().as_millis() as u64,
                    success: false,
                    error: Some(format!("Client error: {}", e)),
                    usage: Usage::new(0, 0),
                },
            };

            // Build provider request
            let provider_messages: Vec<provider_client::Message> = messages.iter().map(|m| {
                provider_client::Message {
                    role: m.role.clone(),
                    content: m.content.clone(),
                }
            }).collect();

            let provider_request = provider_client::ChatRequest {
                provider: provider_slug.clone(),
                model: model.model_id.clone(),
                messages: provider_messages,
                temperature: 0.7,
                max_tokens,
                stream: false,
                extra: std::collections::HashMap::new(),
            };

            // Execute
            match client.chat(provider_request).await {
                Ok(response) => {
                    let latency = start.elapsed().as_millis() as u64;
                    let prompt_tokens = response.usage.get("prompt_tokens").copied().unwrap_or(0);
                    let completion_tokens = response.usage.get("completion_tokens").copied().unwrap_or(0);

                    record_key_result(&state_clone, &provider_slug, key_id, latency as i32, true).await;

                    ModelResult {
                        model: slug.clone(),
                        provider: provider_slug,
                        content: response.message.content,
                        score: 0.0,
                        reason: String::new(),
                        latency_ms: latency,
                        success: true,
                        error: None,
                        usage: Usage::new(prompt_tokens, completion_tokens),
                    }
                }
                Err(e) => {
                    let latency = start.elapsed().as_millis() as u64;
                    record_key_result(&state_clone, &provider_slug, key_id, latency as i32, false).await;

                    ModelResult {
                        model: slug.clone(),
                        provider: provider_slug,
                        content: String::new(),
                        score: 0.0,
                        reason: String::new(),
                        latency_ms: latency,
                        success: false,
                        error: Some(format!("{}", e)),
                        usage: Usage::new(0, 0),
                    }
                }
            }
        });

        tasks.push(task);
    }

    // Await all tasks
    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => results.push(result),
            Err(e) => tracing::error!("Batch task join error: {}", e),
        }
    }

    // 7. Filter successful results for scoring
    let successful: Vec<&ModelResult> = results.iter().filter(|r| r.success).collect();

    if !successful.is_empty() {
        // 8. LLM-as-Judge scoring
        match judge_responses(&state, &user_query, &successful).await {
            Ok(scores) => {
                for result in results.iter_mut() {
                    if let Some(score_info) = scores.iter().find(|s| s.model == result.model) {
                        result.score = score_info.score;
                        result.reason = score_info.reason.clone();
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Judge scoring failed, using default: {}", e);
                for result in results.iter_mut() {
                    if result.success {
                        result.score = (result.content.len() as f64 / 100.0).min(9.0).max(5.0);
                        result.reason = "评分服务不可用".to_string();
                    }
                }
            }
        }
    }

    // 9. Sort by score descending
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    Ok(Json(BatchChatResponse {
        id: format!("batch-{}", uuid::Uuid::new_v4()),
        query: user_query,
        results,
        judge_model: "gpt-4o".to_string(),
        total_latency_ms: total_start.elapsed().as_millis() as u64,
    }))
}

// ── 智能选模型 ─────────────────────────────────────────────────────────────

/// 根据用户问题特征选择 3-4 个最合适的模型
fn smart_select_models(query: &str, models: &[models::LlmModel]) -> Vec<String> {
    let q = query.to_lowercase();

    let code_keywords = ["代码", "编程", "函数", "bug", "debug", "code", "program", "function", "class", "api", "sql", "python", "rust", "java", "javascript", "typescript"];
    let creative_keywords = ["写", "创作", "故事", "诗", "文章", "write", "story", "poem", "creative", "essay", "小说"];
    let analysis_keywords = ["分析", "比较", "评估", "解释", "为什么", "analyze", "compare", "explain", "why", "推理", "逻辑"];

    let is_code = code_keywords.iter().any(|k| q.contains(k));
    let is_creative = creative_keywords.iter().any(|k| q.contains(k));
    let is_analysis = analysis_keywords.iter().any(|k| q.contains(k));

    let mut selected: Vec<String> = Vec::new();
    let mut used_providers: std::collections::HashSet<String> = std::collections::HashSet::new();

    let priority_order: Vec<&str> = if is_code {
        vec!["gpt-4o", "claude-3-5-sonnet", "deepseek-chat"]
    } else if is_creative {
        vec!["claude-3-5-sonnet", "gpt-4o", "gemini-1-5-pro"]
    } else if is_analysis {
        vec!["gpt-4o", "claude-3-5-sonnet", "gemini-1-5-pro"]
    } else {
        vec!["gpt-4o", "claude-3-5-sonnet", "gemini-1-5-pro", "deepseek-chat"]
    };

    for slug in priority_order {
        if selected.len() >= 4 { break; }
        if let Some(model) = models.iter().find(|m| m.slug == slug) {
            if !used_providers.contains(&model.provider_id) {
                selected.push(slug.to_string());
                used_providers.insert(model.provider_id.clone());
            }
        }
    }

    if selected.len() < 3 {
        for model in models {
            if selected.len() >= 4 { break; }
            if !selected.contains(&model.slug) && !used_providers.contains(&model.provider_id) {
                selected.push(model.slug.clone());
                used_providers.insert(model.provider_id.clone());
            }
        }
    }

    selected
}

// ── LLM-as-Judge 评分 ─────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct JudgeScore {
    model: String,
    score: f64,
    reason: String,
}

/// 使用 LLM 对多个回答进行评分
async fn judge_responses(
    state: &Arc<AppState>,
    question: &str,
    results: &[&ModelResult],
) -> Result<Vec<JudgeScore>, ApiError> {
    let mut answers_block = String::new();
    for r in results {
        answers_block.push_str(&format!(
            "---\n{} 的回答：\n{}\n---\n",
            r.model,
            &r.content[..r.content.len().min(2000)]
        ));
    }

    let judge_prompt = format!(
        r#"你是一个AI回答质量评审员。请根据以下标准对每个AI的回答进行评分(1.0-10.0)：
- 准确性：事实是否正确
- 完整性：是否充分回答了问题
- 清晰度：表达是否清楚易懂
- 有用性：对用户是否有实际帮助

用户问题：{}

{}

请以纯JSON数组格式返回评分结果，不要包含其他文字：
[{{"model": "模型名", "score": 8.5, "reason": "简短评语"}}, ...]"#,
        question, answers_block
    );

    let judge_model = state.db.get_model_by_slug("gpt-4o").await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Judge model lookup failed: {}", e)))?
        .ok_or_else(|| ApiError::ModelNotFound("gpt-4o".to_string()))?;

    let provider_slug = judge_model.provider_id.clone();

    let selected_key = {
        let mut scheduler = state.key_scheduler.write().await;
        scheduler.tick();
        scheduler.select_key_no_session(&provider_slug)
    };

    let client = match &selected_key {
        Some(sk) => HttpProviderClient::new_with_key(&provider_slug, &sk.key)
            .map(|c| Arc::new(c) as Arc<dyn ProviderClient>)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?,
        None => HttpProviderClient::new(&provider_slug)
            .map(|c| Arc::new(c) as Arc<dyn ProviderClient>)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("{}", e)))?,
    };

    let judge_request = provider_client::ChatRequest {
        provider: provider_slug.clone(),
        model: judge_model.model_id.clone(),
        messages: vec![
            provider_client::Message::system("你是一个专业的AI回答质量评审员。请严格按照要求的JSON格式返回评分结果。".to_string()),
            provider_client::Message::user(judge_prompt),
        ],
        temperature: 0.3,
        max_tokens: Some(2000),
        stream: false,
        extra: std::collections::HashMap::new(),
    };

    let response = client.chat(judge_request).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Judge call failed: {}", e)))?;

    let content = response.message.content.trim();

    let json_str = if let Some(start) = content.find('[') {
        if let Some(end) = content.rfind(']') {
            &content[start..=end]
        } else {
            content
        }
    } else {
        content
    };

    let scores: Vec<JudgeScore> = serde_json::from_str(json_str)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse judge scores: {} | raw: {}", e, json_str)))?;

    Ok(scores)
}
