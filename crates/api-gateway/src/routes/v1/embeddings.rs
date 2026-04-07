use axum::{Extension, Json};
use crate::error::ApiError;

pub async fn embeddings(
    Extension(_auth): Extension<super::chat::AuthContext>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Err(ApiError::NotImplemented("Embeddings endpoint".to_string()))
}
