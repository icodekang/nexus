//! API v1 版本路由模块
//! 提供统一的 chat 接口以及 OpenAI 和 Anthropic 兼容接口
//!
//! 路由列表：
//! - POST /chat/completions - 统一聊天接口
//! - POST /completions - 文本补全接口（未实现）
//! - POST /embeddings - 向量嵌入接口
//! - GET /models - 模型列表
//! - POST /openai/chat/completions - OpenAI SDK 兼容接口
//! - GET /openai/models - OpenAI SDK 模型列表
//! - POST /anthropic/messages - Anthropic SDK 兼容接口
//! - GET /anthropic/models - Anthropic SDK 模型列表

pub mod anthropic;
pub mod chat;
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
        .route(
            "/openai/chat/completions",
            post(openai::openai_chat_completions),
        )
        .route(
            "/openai/models",
            axum::routing::get(openai::openai_list_models),
        )
        .route("/anthropic/messages", post(anthropic::anthropic_messages))
        .route(
            "/anthropic/models",
            axum::routing::get(anthropic::anthropic_list_models),
        )
}
