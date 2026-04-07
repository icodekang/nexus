pub mod chat;
pub mod models;
pub mod embeddings;

use axum::{routing::post, Router};
use std::sync::Arc;

pub struct RouterContext;

pub fn routes() -> Router<Arc<RouterContext>> {
    Router::new()
        .route("/chat/completions", post(chat::chat_completions))
        .route("/completions", post(chat::completions))
        .route("/embeddings", post(embeddings::embeddings))
        .route("/models", axum::routing::get(models::list_models))
}
