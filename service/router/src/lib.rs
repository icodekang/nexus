pub mod selector;
pub mod strategy;
pub mod context;
pub mod error;

pub use selector::*;
pub use strategy::*;
pub use context::*;
pub use error::*;

use models::{Provider, LlmModel, BuiltinModels, Providers};

/// Router core for selecting the best provider
pub struct RouterCore {
    providers: Vec<Provider>,
    models: Vec<LlmModel>,
}

impl RouterCore {
    pub fn new() -> Self {
        let providers = Providers::all();
        let models = BuiltinModels::all();
        
        Self { providers, models }
    }

    /// Select the best provider for a given model
    pub async fn select_provider(
        &self,
        model_slug: &str,
        strategy: RouteStrategy,
    ) -> Result<Provider, RouterError> {
        // Find the model
        let model = self.models.iter()
            .find(|m| m.slug == model_slug)
            .ok_or(RouterError::ModelNotFound(model_slug.to_string()))?;

        // Find providers that support this model
        let candidates: Vec<&Provider> = self.providers.iter()
            .filter(|p| p.slug == model.provider_id && p.is_active)
            .collect();

        if candidates.is_empty() {
            return Err(RouterError::NoProviderAvailable);
        }

        // Select using the given strategy
        selector::select(&model, &candidates, strategy)
    }

    /// Get all available models grouped by provider
    pub fn get_models_by_provider(&self) -> std::collections::HashMap<String, Vec<&LlmModel>> {
        let mut result: std::collections::HashMap<String, Vec<&LlmModel>> = std::collections::HashMap::new();
        
        for model in &self.models {
            result
                .entry(model.provider_id.clone())
                .or_default()
                .push(model);
        }
        
        result
    }

    /// Get model by slug
    pub fn get_model(&self, slug: &str) -> Option<&LlmModel> {
        self.models.iter().find(|m| m.slug == slug)
    }
}

impl Default for RouterCore {
    fn default() -> Self {
        Self::new()
    }
}
