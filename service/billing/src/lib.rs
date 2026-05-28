//! 计费模块
//!
//! 按 Token 计费引擎，支持多维度定价

pub mod error;

pub use error::*;

use models::{CostBreakdown, ModelPricing, PricingMode};
use rust_decimal::Decimal;

pub struct BillingEngine;

impl BillingEngine {
    pub fn new() -> Self {
        Self
    }

    /// 计算单次调用的费用 = Σ(维度价格 × 维度用量)
    pub fn calculate(
        pricing: &ModelPricing,
        input_tokens: i32,
        output_tokens: i32,
        reasoning_tokens: i32,
        image_count: i32,
        cache_read_tokens: i32,
    ) -> CostBreakdown {
        let mut b = CostBreakdown::default();

        match pricing.pricing_mode {
            PricingMode::PerToken => {
                b.prompt_cost =
                    pricing.prompt_price * Decimal::from(input_tokens);
                b.completion_cost =
                    pricing.completion_price * Decimal::from(output_tokens);
                if let Some(ip) = pricing.image_price {
                    b.image_cost = ip * Decimal::from(image_count);
                }
                if let Some(rp) = pricing.reasoning_price {
                    b.reasoning_cost = rp * Decimal::from(reasoning_tokens);
                }
                if let Some(cp) = pricing.cache_read_price {
                    b.cache_read_cost = cp * Decimal::from(cache_read_tokens);
                }
                b.request_cost = pricing.request_price.unwrap_or_default();
            }
            PricingMode::PerRequest => {
                let total_tokens = input_tokens + output_tokens;
                let avg = Decimal::from(pricing.avg_tokens_per_request);
                if !avg.is_zero() {
                    let ratio = Decimal::from(total_tokens) / avg;
                    let request = pricing.request_price.unwrap_or_default();
                    let equivalent = request * ratio;
                    let capped = equivalent.min(request);
                    let total = Decimal::from(total_tokens.max(1));
                    let input_ratio = Decimal::from(input_tokens) / total;
                    b.prompt_cost = capped * input_ratio;
                    b.completion_cost = capped - b.prompt_cost;
                }
            }
        }
        b.total = b.sum();
        b
    }
}

impl Default for BillingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn test_pricing() -> ModelPricing {
        ModelPricing {
            id: uuid::Uuid::new_v4(),
            model_slug: "gpt-4o".into(),
            provider_slug: "openai".into(),
            prompt_price: Decimal::from_str("0.0000025").unwrap(),
            completion_price: Decimal::from_str("0.000010").unwrap(),
            image_price: None,
            reasoning_price: None,
            cache_read_price: None,
            request_price: None,
            pricing_mode: PricingMode::PerToken,
            avg_tokens_per_request: 5000,
            effective_from: chrono::Utc::now(),
            effective_until: None,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_per_token_calculation() {
        let pricing = test_pricing();
        let cost = BillingEngine::calculate(&pricing, 1000, 500, 0, 0, 0);
        assert_eq!(cost.prompt_cost, Decimal::from_str("0.0025").unwrap());
        assert_eq!(cost.completion_cost, Decimal::from_str("0.005").unwrap());
        assert_eq!(cost.total, Decimal::from_str("0.0075").unwrap());
    }

    #[test]
    fn test_free_with_zero_usage() {
        let pricing = test_pricing();
        let cost = BillingEngine::calculate(&pricing, 0, 0, 0, 0, 0);
        assert!(cost.total.is_zero());
    }
}
