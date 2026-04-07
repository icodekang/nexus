use axum::{routing::{get, post, delete}, Router, Extension, Json};
use super::chat::AuthContext;

pub fn routes() -> Router {
    Router::new()
        .route("/balance", get(balance))
        .route("/usage", get(usage))
        .route("/keys", get(list_keys).post(create_key))
        .route("/keys/:key_id", delete(delete_key))
}

async fn balance(Extension(auth): Extension<AuthContext>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "credits": auth.credits.to_string(),
        "currency": "USD"
    }))
}

async fn usage() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "total_usage": "0.00",
        "usage_by_model": {}
    }))
}

async fn list_keys() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "data": []
    }))
}

async fn create_key() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": "key_123",
        "key": "sk-nova-xxxx",
        "name": "My API Key"
    }))
}

async fn delete_key() -> Json<serde_json::Value> {
    Json(serde_json::json!({"deleted": true}))
}
