pub mod chat;
pub mod models;

use axum::{routing::post, Router};
use std::sync::Arc;

use crate::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/chat/completions", post(chat::chat_completions))
        .route("/completions", post(chat::completions))
        .route("/embeddings", post(chat::embeddings))
        .route("/models", axum::routing::get(models::list_models))
}
