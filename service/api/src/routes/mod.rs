pub mod v1;
pub mod auth;
pub mod me;
pub mod admin;
pub mod openai;

use axum::{http::StatusCode, Json};

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "nexus-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
