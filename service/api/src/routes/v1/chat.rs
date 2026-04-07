use axum::{
    extract::{State, Extension, Query},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;
use crate::middleware::auth::AuthContext;
use models::{ChatRequest, ChatResponse, Message, Usage, ModelWithProvider};

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
) -> Result<impl IntoResponse, ApiError> {
    let is_stream = query.stream.unwrap_or(false);

    // Get the model - could be "provider/model" format or just "model"
    let (provider_slug, model_slug) = parse_model_string(&request.model);
    
    // Look up the model
    let model = state.db.get_model_by_slug(&model_slug)
        .await
        .map_err(|e| ApiError::Internal(e))?
        .ok_or_else(|| ApiError::ModelNotFound(request.model.clone()))?;

    // Check if model supports streaming if requested
    if is_stream && !model.capabilities.contains(&"streaming".to_string()) {
        return Err(ApiError::InvalidRequest(
            "Model does not support streaming".to_string()
        ));
    }

    if is_stream {
        // TODO: Implement streaming response
        return Err(ApiError::NotImplemented("Streaming not implemented yet".to_string()));
    }

    // For now, return a mock response
    // In real implementation, this would call the router to select a provider
    // and forward the request to the appropriate adapter
    let response = ChatResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp() as u64,
        model: request.model.clone(),
        choices: vec![models::Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: "This is a mock response. In production, this would be the actual LLM response.".to_string(),
                name: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
    };

    Ok(Json(response))
}

/// POST /v1/completions
pub async fn completions(
    State(_state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Json(_request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ApiError> {
    Err(ApiError::NotImplemented("Completions endpoint not implemented. Use /v1/chat/completions".to_string()))
}

/// POST /v1/embeddings
pub async fn embeddings(
    State(_state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Json(_request): Json<serde_json::Value>,
) -> Result<impl IntoResponse, ApiError> {
    Err(ApiError::NotImplemented("Embeddings not implemented yet".to_string()))
}

/// Parse model string that could be "provider/model" or just "model"
fn parse_model_string(model_str: &str) -> (String, String) {
    if let Some((provider, model)) = model_str.split_once('/') {
        (provider.to_string(), model.to_string())
    } else {
        (String::new(), model_str.to_string())
    }
}
