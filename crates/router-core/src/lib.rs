pub mod selector;
pub mod strategy;
pub mod context;
pub mod fallback;
pub mod error;

use models::llm::{ChatRequest, ChatResponse, Model};
use models::provider::Provider;
use crate::error::RouterError;

/// Core router that selects the best provider for a request
pub struct RouterCore {
    providers: Vec<Provider>,
}

impl RouterCore {
    pub fn new() -> Self {
        Self {
            providers: Provider::list_all(),
        }
    }

    /// Select the best provider based on strategy
    pub async fn select_provider(
        &self,
        model: &Model,
        strategy: selector::RouteStrategy,
    ) -> Result<Provider, RouterError> {
        selector::select(model, &self.providers, strategy).await
    }

    /// Forward request to selected provider via gRPC to Python adapter
    pub async fn forward_to_provider(
        &self,
        provider: &Provider,
        _request: ChatRequest,
    ) -> Result<ChatResponse, RouterError> {
        tracing::info!("Forwarding to provider: {}", provider.name);
        Err(RouterError::AdapterNotImplemented)
    }
}

impl Default for RouterCore {
    fn default() -> Self {
        Self::new()
    }
}
