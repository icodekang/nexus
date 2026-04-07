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

impl RouteStrategy {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cheapest" => RouteStrategy::Cheapest,
            "fastest" => RouteStrategy::Fastest,
            "quality" => RouteStrategy::Quality,
            "balanced" => RouteStrategy::Balanced,
            _ => RouteStrategy::Balanced, // Default
        }
    }
}

impl Default for RouteStrategy {
    fn default() -> Self {
        RouteStrategy::Balanced
    }
}
