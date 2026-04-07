use models::llm::Model;
use models::provider::Provider;
use crate::error::RouterError;

pub struct BalancedStrategy;

impl BalancedStrategy {
    pub fn select(
        providers: &[&Provider],
        _model: &Model,
    ) -> Result<Provider, RouterError> {
        // Balanced = combination of price, latency, quality
        // Score = 0.4 * normalized_price + 0.4 * normalized_latency + 0.2 * normalized_quality
        // For now, just use priority
        let mut sorted = providers.to_vec();
        sorted.sort_by_key(|p| p.priority);

        sorted.first()
            .map(|p| (*p).clone())
            .ok_or(RouterError::NoProviderAvailable)
    }
}
