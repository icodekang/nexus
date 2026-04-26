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

pub mod chat;
pub mod models;
pub mod anthropic;
pub mod openai;
pub mod shared;

/// 重新导出共享工具函数
pub use shared::{
    rate_limit_for_plan, select_key, create_client, record_result,
    extract_session_id, default_session_id,
    check_subscription, check_token_quota,
    validate_temperature, add_rate_limit_headers, log_api_call,
};

use axum::{routing::post, Router};
use std::sync::Arc;

use crate::state::AppState;

/// 创建 v1 路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/chat/completions", post(chat::chat_completions))
        .route("/chat/batch", post(chat::chat_batch))
        .route("/completions", post(chat::completions))
        .route("/embeddings", post(chat::embeddings))
        .route("/models", axum::routing::get(models::list_models))
        .route("/openai/chat/completions", post(openai::openai_chat_completions))
        .route("/openai/models", axum::routing::get(openai::openai_list_models))
        .route("/anthropic/messages", post(anthropic::anthropic_messages))
        .route("/anthropic/models", axum::routing::get(anthropic::anthropic_list_models))
}
