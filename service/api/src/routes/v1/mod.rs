pub mod chat;
pub mod models;
pub mod anthropic;
pub mod openai;
pub mod shared;

pub use shared::{
    rate_limit_for_plan, select_key, create_client, record_result,
    extract_session_id, default_session_id,
    check_subscription, check_token_quota,
    validate_temperature, add_rate_limit_headers, log_api_call,
};

use axum::{routing::post, Router};
use std::sync::Arc;

use crate::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/chat/completions", post(chat::chat_completions))
        .route("/completions", post(chat::completions))
        .route("/embeddings", post(chat::embeddings))
        .route("/models", axum::routing::get(models::list_models))
        .route("/openai/chat/completions", post(openai::openai_chat_completions))
        .route("/openai/models", axum::routing::get(openai::openai_list_models))
        .route("/anthropic/messages", post(anthropic::anthropic_messages))
        .route("/anthropic/models", axum::routing::get(anthropic::anthropic_list_models))
}
