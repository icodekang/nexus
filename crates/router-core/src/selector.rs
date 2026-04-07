use models::llm::Model;
use models::provider::Provider;
use crate::error::RouterError;
use super::strategy::{CheapestStrategy, FastestStrategy, QualityStrategy, BalancedStrategy};

/// Routing strategy
#[derive(Debug, Clone, Copy)]
pub enum RouteStrategy {
    /// Select the cheapest provider
    Cheapest,
    /// Select the fastest provider (lowest latency)
    Fastest,
    /// Select the highest quality (largest context window)
    Quality,
    /// Balanced score (price + latency + quality)
    Balanced,
}

/// Select the best provider based on strategy
pub async fn select(
    model: &Model,
    providers: &[Provider],
    strategy: RouteStrategy,
) -> Result<Provider, RouterError> {
    let candidates: Vec<_> = providers.iter()
        .filter(|p| p.is_active)
        .collect();

    if candidates.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    match strategy {
        RouteStrategy::Cheapest => CheapestStrategy::select(&candidates, model),
        RouteStrategy::Fastest => FastestStrategy::select(&candidates, model),
        RouteStrategy::Quality => QualityStrategy::select(&candidates, model),
        RouteStrategy::Balanced => BalancedStrategy::select(&candidates, model),
    }
}
