use axum::{extract::Query, Json};
use serde::Deserialize;
use models::llm::Model;

#[derive(Debug, Deserialize)]
pub struct ModelsQuery {
    provider: Option<String>,
}

pub async fn list_models(
    Query(query): Query<ModelsQuery>,
) -> Json<serde_json::Value> {
    let models = if let Some(provider) = &query.provider {
        Model::list_by_provider(provider)
    } else {
        Model::list_all()
    };

    Json(serde_json::json!({
        "object": "list",
        "data": models.into_iter().map(|m| m.to_api_response()).collect::<Vec<_>>()
    }))
}
