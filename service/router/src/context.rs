use crate::RouteStrategy;

/// Context for routing a request
#[derive(Debug, Clone)]
pub struct RouteContext {
    pub model: String,
    pub strategy: RouteStrategy,
    pub provider_hint: Option<String>,
}

impl RouteContext {
    pub fn new(model: String) -> Self {
        Self {
            model,
            strategy: RouteStrategy::default(),
            provider_hint: None,
        }
    }

    pub fn with_strategy(mut self, strategy: RouteStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_provider_hint(mut self, provider: String) -> Self {
        self.provider_hint = Some(provider);
        self
    }
}
