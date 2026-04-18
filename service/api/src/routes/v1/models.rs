//! 模型列表路由模块
//! 提供模型列表查询接口

use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::AppState;
use crate::error::ApiError;
use models::{ModelWithProvider, Provider};

/// 模型查询参数
#[derive(Debug, Deserialize)]
pub struct ModelsQuery {
    /// 按 Provider 过滤（可选）
    pub provider: Option<String>,
}

/// GET /v1/models
///
/// 获取可用模型列表
///
/// # 说明
/// 支持按 Provider 过滤，结果会被缓存
///
/// # 参数
/// * `Query(query)` - 查询参数，可指定 provider 过滤
///
/// # 返回
/// - object: "list"
/// - data: 模型列表，每项包含 id、object、created、owned_by、provider 等信息
pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ModelsQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Try to get from cache first
    if let Ok(Some(cached)) = state.redis.get_cached_models().await {
        if let Ok(all_models) = serde_json::from_str::<Vec<ModelWithProvider>>(&cached) {
            let filtered = filter_models(all_models, &query.provider);
            return Ok(Json(serde_json::json!({
                "object": "list",
                "data": filtered
            })));
        }
    }

    // Load providers from database
    let providers = state.db.list_providers().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to load providers: {}", e)))?;

    let provider_map: std::collections::HashMap<String, &Provider> = providers
        .iter()
        .map(|p| (p.slug.clone(), p))
        .collect();

    // Load models from database
    let all_models = if let Some(ref provider_slug) = query.provider {
        state.db.list_models_by_provider(provider_slug).await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to load models: {}", e)))?
    } else {
        state.db.list_models().await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to load models: {}", e)))?
    };

    // Convert to ModelWithProvider
    let models_with_providers: Vec<ModelWithProvider> = all_models
        .iter()
        .filter_map(|m| {
            provider_map.get(&m.provider_id)
                .map(|p| ModelWithProvider::from_model(m, p))
        })
        .collect();

    // Cache the unfiltered result (only if no provider filter)
    if query.provider.is_none() {
        if let Ok(json) = serde_json::to_string(&models_with_providers) {
            let _ = state.redis.cache_models(&json).await;
        }
    }

    Ok(Json(serde_json::json!({
        "object": "list",
        "data": models_with_providers
    })))
}

fn filter_models(models: Vec<ModelWithProvider>, provider: &Option<String>) -> Vec<ModelWithProvider> {
    match provider {
        Some(p) => models.into_iter().filter(|m| m.provider == *p).collect(),
        None => models,
    }
}
