pub mod v1;
pub mod auth;
pub mod me;
pub mod admin;

use axum::Json;

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "nexus-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
