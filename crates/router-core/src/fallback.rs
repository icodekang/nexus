use models::llm::Model;
use models::provider::Provider;
use crate::error::RouterError;
use super::selector::RouteStrategy;

/// Fallback provider selection when primary fails
pub struct FallbackRouter {
    primary: Provider,
    fallbacks: Vec<Provider>,
}

impl FallbackRouter {
    pub fn new(primary: Provider) -> Self {
        Self {
            primary,
            fallbacks: vec![],
        }
    }

    pub fn with_fallback(mut self, provider: Provider) -> Self {
        self.fallbacks.push(provider);
        self
    }

    /// Try primary, then fallbacks in order
    pub async fn route(
        &self,
        model: &Model,
        strategy: RouteStrategy,
    ) -> Result<Provider, RouterError> {
        // Try primary
        if self.is_available(&self.primary).await {
            return Ok(self.primary.clone());
        }

        // Try fallbacks
        for provider in &self.fallbacks {
            if self.is_available(provider).await {
                return Ok(provider.clone());
            }
        }

        Err(RouterError::NoProviderAvailable)
    }

    async fn is_available(&self, _provider: &Provider) -> bool {
        // TODO: Check provider health / availability
        true
    }
}
