use models::{Provider, LlmModel};

use crate::{RouteStrategy, RouterError};

/// Select the best provider based on strategy
pub fn select(
    model: &LlmModel,
    providers: &[&Provider],
    strategy: RouteStrategy,
) -> Result<Provider, RouterError> {
    if providers.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    // Filter to only active providers
    let active: Vec<&Provider> = providers.iter()
        .filter(|p| p.is_active)
        .cloned()
        .collect();

    if active.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    match strategy {
        RouteStrategy::Cheapest => {
            // Sort by priority (lower priority = higher preference = cheaper)
            // In a real system, this would use actual pricing data
            let mut sorted = active.clone();
            sorted.sort_by_key(|p| p.priority);
            Ok(sorted.first().unwrap().clone())
        }
        RouteStrategy::Fastest => {
            // Sort by priority as a proxy for latency
            // In a real system, this would use actual latency measurements
            let mut sorted = active.clone();
            sorted.sort_by_key(|p| p.priority);
            Ok(sorted.first().unwrap().clone())
        }
        RouteStrategy::Quality => {
            // Higher priority = higher quality (larger context window in our model)
            let mut sorted = active.clone();
            sorted.sort_by(|a, b| {
                // Higher priority number = better quality
                b.priority.cmp(&a.priority)
            });
            Ok(sorted.first().unwrap().clone())
        }
        RouteStrategy::Balanced => {
            // Balanced uses priority as a composite score
            // In a real system, this would combine price, latency, and quality
            let mut sorted = active.clone();
            sorted.sort_by_key(|p| p.priority);
            Ok(sorted.first().unwrap().clone())
        }
    }
}

/// Select provider with fallback (if first provider fails, try next)
pub fn select_with_fallback(
    model: &LlmModel,
    providers: &[&Provider],
    strategy: RouteStrategy,
) -> Result<Vec<Provider>, RouterError> {
    if providers.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    // Filter to only active providers
    let mut active: Vec<Provider> = providers.iter()
        .filter(|p| p.is_active)
        .cloned()
        .cloned()
        .collect();

    if active.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    match strategy {
        RouteStrategy::Cheapest | RouteStrategy::Fastest | RouteStrategy::Balanced => {
            active.sort_by_key(|p| p.priority);
        }
        RouteStrategy::Quality => {
            active.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
    }

    Ok(active)
}
