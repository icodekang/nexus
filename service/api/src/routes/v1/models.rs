use axum::{extract::Query, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::AppState;
use crate::error::ApiError;
use models::{ModelWithProvider, BuiltinModels, Provider, Providers};

#[derive(Debug, Deserialize)]
pub struct ModelsQuery {
    pub provider: Option<String>,
}

/// GET /v1/models
pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ModelsQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Try to get from cache first
    if let Ok(Some(cached)) = state.redis.get_cached_models().await {
        if let Ok(models) = serde_json::from_str::<Vec<ModelWithProvider>>(&cached) {
            return Ok(Json(serde_json::json!({
                "object": "list",
                "data": models.into_iter().map(|m| serde_json::to_value(&m).unwrap()).collect::<Vec<_>>()
            })));
        }
    }

    // Get all providers
    let providers: Vec<Provider> = Providers::all();
    let provider_map: std::collections::HashMap<String, Provider> = providers
        .into_iter()
        .map(|p| (p.slug.clone(), p))
        .collect();

    // Get all models
    let all_models = BuiltinModels::all();
    
    // Convert to ModelWithProvider
    let models_with_providers: Vec<ModelWithProvider> = all_models
        .iter()
        .filter_map(|m| {
            provider_map.get(&m.provider_id)
                .map(|p| ModelWithProvider::from_model(m, p))
        })
        .collect();

    // Filter by provider if specified
    let filtered_models = if let Some(provider) = &query.provider {
        models_with_providers
            .into_iter()
            .filter(|m| m.provider == *provider)
            .collect()
    } else {
        models_with_providers
    };

    // Cache the result
    if let Ok(json) = serde_json::to_string(&filtered_models) {
        let _ = state.redis.cache_models(&json).await;
    }

    Ok(Json(serde_json::json!({
        "object": "list",
        "data": filtered_models.into_iter().map(|m| serde_json::to_value(&m).unwrap()).collect::<Vec<_>>()
    })))
}
