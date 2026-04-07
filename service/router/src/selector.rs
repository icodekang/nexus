use models::{Provider, LlmModel};

use crate::{RouteStrategy, RouterError};

/// Select the best provider based on strategy
pub fn select(
    _model: &LlmModel,
    providers: &[&Provider],
    strategy: RouteStrategy,
) -> Result<Provider, RouterError> {
    if providers.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    match strategy {
        RouteStrategy::Cheapest => {
            // Sort by priority (lower = more preferred = cheaper)
            let mut sorted = providers.to_vec();
            sorted.sort_by_key(|p| p.priority);
            Ok(sorted.first().unwrap().clone())
        }
        RouteStrategy::Fastest => {
            // For now, just use priority as a proxy for speed
            let mut sorted = providers.to_vec();
            sorted.sort_by_key(|p| p.priority);
            Ok(sorted.first().unwrap().clone())
        }
        RouteStrategy::Quality => {
            // Quality is determined by priority (higher priority = better quality)
            let mut sorted = providers.to_vec();
            sorted.sort_by(|a, b| a.priority.cmp(&b.priority).reverse());
            Ok(sorted.first().unwrap().clone())
        }
        RouteStrategy::Balanced => {
            // Balanced just uses priority
            let mut sorted = providers.to_vec();
            sorted.sort_by_key(|p| p.priority);
            Ok(sorted.first().unwrap().clone())
        }
    }
}
