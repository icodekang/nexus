use models::llm::Model;
use models::provider::Provider;
use crate::error::RouterError;

pub struct QualityStrategy;

impl QualityStrategy {
    pub fn select(
        providers: &[&Provider],
        model: &Model,
    ) -> Result<Provider, RouterError> {
        // Quality = context window size (higher is better)
        // In real impl, use actual model metadata
        let _context_window = model.context_window;

        let mut sorted = providers.to_vec();
        sorted.sort_by_key(|p| p.priority);

        sorted.first()
            .map(|p| (*p).clone())
            .ok_or(RouterError::NoProviderAvailable)
    }
}
