use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json, Extension,
};
use std::sync::Arc;
use futures_util::Stream;
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use serde::Deserialize;

use crate::error::ApiError;
use crate::state::AppState;
use crate::middleware::auth::AuthContext;
use models::{ChatRequest, ChatResponse, Message, Usage, User, SubscriptionPlan};
use provider_client::{
    ChatRequest as ProviderChatRequest,
    ProviderClient, ProviderClientFactory,
};

// Rate limit defaults per plan (requests per minute)
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

#[derive(Debug, Deserialize)]
pub struct ChatQuery {
    pub stream: Option<bool>,
}

/// POST /v1/chat/completions
pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ChatQuery>,
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

    // 2. Parse model string to get provider and model
    let (provider_slug, model_slug) = parse_model_string(&request.model);

    // 3. Look up the model
    let model = state.db.get_model_by_slug(&model_slug)
        .await
        .map_err(ApiError::Internal)?
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
    // Add other active providers as fallback
    if let Ok(all_providers) = state.db.list_providers().await {
        for p in &all_providers {
            if p.is_active && p.slug != provider_for_request {
                providers_to_try.push(p.slug.clone());
            }
        }
    }

    // Build request for provider client
    let provider_request = ProviderChatRequest {
        provider: provider_for_request.clone(),
        model: model.model_id.clone(),
        messages: request.messages.clone(),
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        stream: is_stream,
        extra: std::collections::HashMap::new(),
    };

    let start_time = std::time::Instant::now();

    if is_stream {
        // Handle streaming response
        let (tx, rx) = broadcast::channel::<Result<Event, std::convert::Infallible>>(1024);

        let model_name = request.model.clone();
        let state_clone = state.clone();
        let auth_clone = auth.clone();
        let provider_clone = provider_for_request.clone();
        let model_slug_clone = model.slug.clone();

        tokio::spawn(async move {
            let mut total_input_tokens: i32 = 0;
            let mut total_output_tokens: i32 = 0;

            match provider_client.chat_stream(provider_request).await {
                Ok(chunks) => {
                    for chunk in chunks {
                        // Track tokens roughly (count from delta)
                        if !chunk.delta.is_empty() {
                            total_output_tokens += 1; // rough estimate
                        }

                        let event = if chunk.finished {
                            let chat_chunk = models::ChatChunk::new(
                                &model_name,
                                "",
                                true,
                            );
                            Event::default()
                                .event("message")
                                .data(serde_json::to_string(&chat_chunk).unwrap_or_default())
                        } else {
                            let chat_chunk = models::ChatChunk::new(
                                &model_name,
                                &chunk.delta,
                                false,
                            );
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
                    let _ = tx.send(Ok(Event::default()
                        .event("error")
                        .data(format!("{{\"error\": \"{}\"}}", e))));
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
                total_input_tokens,
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
        // Handle non-streaming response with fallback
        let mut last_error = None;
        let mut provider_resp = None;
        let mut used_provider = provider_for_request.clone();

        for try_provider in &providers_to_try {
            match ProviderClientFactory::create(try_provider) {
                Ok(client) => {
                    let mut req = provider_request.clone();
                    req.provider = try_provider.clone();
                    match client.chat(req).await {
                        Ok(resp) => {
                            provider_resp = Some(resp);
                            used_provider = try_provider.clone();
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("Provider {} failed: {}", try_provider, e);
                            last_error = Some(e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to create client for {}: {}", try_provider, e);
                    last_error = Some(e);
                }
            }
        }

        let provider_resp = provider_resp.ok_or_else(|| {
            ApiError::ProviderError(format!("All providers failed. Last error: {:?}", last_error))
        })?;

        let latency_ms = start_time.elapsed().as_millis() as i32;

        // Convert provider response to OpenAI-compatible format
        let prompt_tokens = provider_resp.usage.get("prompt_tokens").copied().unwrap_or(0) as i32;
        let completion_tokens = provider_resp.usage.get("completion_tokens").copied().unwrap_or(0) as i32;
        let total_tokens = provider_resp.usage.get("total_tokens").copied()
            .unwrap_or((prompt_tokens + completion_tokens) as u64) as i32;

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

        // Log the API call
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

/// Add rate limit headers to response (OpenRouter-compatible)
fn add_rate_limit_headers(response: &mut Response, limit: i64, remaining: i64, reset: i64) {
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", limit.to_string().parse().unwrap());
    headers.insert("X-RateLimit-Remaining", remaining.to_string().parse().unwrap());
    headers.insert("X-RateLimit-Reset", reset.to_string().parse().unwrap());
}

/// Check if user's subscription is active
fn check_subscription(user: &User) -> Result<(), ApiError> {
    match user.subscription_plan {
        SubscriptionPlan::None => {
            // Free tier is always "active" but has limited quota
        }
        SubscriptionPlan::Monthly | SubscriptionPlan::Yearly | SubscriptionPlan::Team | SubscriptionPlan::Enterprise => {
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

/// Check if user has remaining token quota in current billing period
async fn check_token_quota(state: &AppState, user: &User) -> Result<(), ApiError> {
    let quota = user.subscription_plan.monthly_token_quota();
    if quota == i64::MAX {
        return Ok(()); // Enterprise: unlimited
    }

    // Determine billing period
    let now = chrono::Utc::now();
    let period_start = user.subscription_start.unwrap_or(now);
    let period_end = user.subscription_end.unwrap_or(
        now + chrono::Duration::days(user.subscription_plan.billing_cycle_days())
    );

    // If current time is past the period end, the period has expired
    // (subscription check should have caught this, but handle edge case)
    if now > period_end {
        return Err(ApiError::SubscriptionExpired);
    }

    // Query current period usage
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

/// Log API call to database
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
pub async fn completions(
    State(_state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Json(_request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ApiError> {
    Err(ApiError::NotImplemented(
        "Completions endpoint not implemented. Use /v1/chat/completions".to_string()
    ))
}

/// POST /v1/embeddings
pub async fn embeddings(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<models::EmbeddingsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Check subscription
    check_subscription(&auth.user)?;

    // Rate limiting
    let user_id = auth.user.id.to_string();
    let rpm = rate_limit_for_plan(&auth.user.subscription_plan);
    let (allowed, remaining, reset_time) = state.redis
        .check_rate_limit(&user_id, rpm, 60)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Rate limit check failed: {}", e)))?;

    if !allowed {
        return Err(ApiError::RateLimitExceeded);
    }

    // Get provider from model name
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

    // Log the API call
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

/// Parse model string that could be "provider/model" or just "model"
fn parse_model_string(model_str: &str) -> (String, String) {
    if let Some((provider, model)) = model_str.split_once('/') {
        (provider.to_string(), model.to_string())
    } else {
        (String::new(), model_str.to_string())
    }
}

/// Validate chat request parameters
fn validate_chat_request(request: &ChatRequest, context_window: i32) -> Result<(), ApiError> {
    // Validate temperature (0.0 - 2.0)
    if request.temperature < 0.0 || request.temperature > 2.0 {
        return Err(ApiError::InvalidRequest(
            "temperature must be between 0.0 and 2.0".to_string()
        ));
    }

    // Validate max_tokens
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

    // Validate top_p
    if let Some(top_p) = request.top_p {
        if top_p < 0.0 || top_p > 1.0 {
            return Err(ApiError::InvalidRequest(
                "top_p must be between 0.0 and 1.0".to_string()
            ));
        }
    }

    // Validate messages are not empty
    if request.messages.is_empty() {
        return Err(ApiError::InvalidRequest(
            "messages must not be empty".to_string()
        ));
    }

    // Validate message roles
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
