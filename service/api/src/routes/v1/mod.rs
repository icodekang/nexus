//! API v1 版本路由模块
//!
//! 路由分组：
//!   /v1/chat/*          — 统一聊天接口
//!   /v1/messages        — HTTP 协议端点（Anthropic Messages 格式）
//!   /v1/openai/*        — OpenAI SDK 兼容路由
//!   /v1/anthropic/*     — Anthropic SDK 兼容路由

pub mod anthropic;
pub mod chat;
pub mod messages;
pub mod models;
pub mod openai;
pub mod shared;

/// 重新导出共享工具函数
pub use shared::{
    add_rate_limit_headers, charge_tokens, check_balance, create_client,
    create_client_from_source, default_session_id, extract_session_id, log_api_call,
    rate_limit_for_user, record_result, select_key_with_priority, validate_temperature,
    SelectedKeySource,
};

use axum::{routing::post, Router};
use std::sync::Arc;

use crate::state::AppState;

/// 创建 v1 路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/chat/completions", post(chat::chat_completions))
        .route("/chat/batch", post(chat::chat_batch))
        .route("/chat/batch/judge", post(chat::chat_batch_judge))
        .route("/completions", post(chat::completions))
        .route("/embeddings", post(chat::embeddings))
        .route("/messages", post(messages::messages_endpoint))
        .nest("/openai", openai_routes())
        .nest("/anthropic", anthropic_routes())
}

fn openai_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(openai::openai_chat_completions))
        .route(
            "/chat/completions",
            post(openai::openai_chat_completions),
        )
        .route(
            "/models",
            axum::routing::get(openai::openai_list_models),
        )
}

fn anthropic_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(anthropic::anthropic_messages))
        .route(
            "/models",
            axum::routing::get(anthropic::anthropic_list_models),
        )
}
