use axum::{
    extract::{State, Extension, Query},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::error::ApiError;
use models::llm::{ChatRequest, ChatResponse, Usage};
use super::RouterContext;

#[derive(Debug, serde::Deserialize)]
pub struct ChatQuery {
    pub stream: Option<bool>,
}

pub async fn chat_completions(
    State(_ctx): State<Arc<RouterContext>>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<ChatQuery>,
    Json(request): Json<ChatRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let is_stream = query.stream.unwrap_or(false);

    // Validate model
    let model = models::llm::Model::get_by_id(&request.model)
        .ok_or_else(|| ApiError::ModelNotFound(request.model.clone()))?;

    // Check credits
    if auth.credits < model.estimated_cost() {
        return Err(ApiError::InsufficientCredits {
            required: model.estimated_cost().to_string(),
            available: auth.credits.to_string(),
        });
    }

    if is_stream {
        // TODO: Implement streaming
        Ok(Json(serde_json::json!({"error": "streaming not implemented"})))
    } else {
        Ok(Json(ChatResponse {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: request.model.clone(),
            choices: vec![],
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
        }))
    }
}

pub async fn completions(
    State(_ctx): State<Arc<RouterContext>>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Err(ApiError::NotImplemented("Completions endpoint".to_string()))
}

pub async fn embeddings(
    State(_ctx): State<Arc<RouterContext>>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Err(ApiError::NotImplemented("Embeddings endpoint".to_string()))
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub credits: rust_decimal::Decimal,
}
