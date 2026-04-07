pub mod v1;
pub mod auth;
pub mod me;

use axum::{http::StatusCode, Json};

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "api-gateway",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
