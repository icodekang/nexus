use models::llm::ChatRequest;
use super::selector::RouteStrategy;

/// Context for routing a request
#[derive(Debug, Clone)]
pub struct RouteContext {
    pub request: ChatRequest,
    pub strategy: RouteStrategy,
    pub max_latency_ms: Option<u32>,
    pub max_price: Option<f64>,
    pub required_capabilities: Vec<String>,
}

impl RouteContext {
    pub fn new(request: ChatRequest) -> Self {
        Self {
            request,
            strategy: RouteStrategy::Balanced,
            max_latency_ms: None,
            max_price: None,
            required_capabilities: vec![],
        }
    }

    pub fn with_strategy(mut self, strategy: RouteStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_max_latency(mut self, ms: u32) -> Self {
        self.max_latency_ms = Some(ms);
        self
    }

    pub fn with_max_price(mut self, price: f64) -> Self {
        self.max_price = Some(price);
        self
    }
}
