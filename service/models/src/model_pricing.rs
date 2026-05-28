//! 模型定价

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub id: Uuid,
    pub model_slug: String,
    pub provider_slug: String,

    pub prompt_price: Decimal,
    pub completion_price: Decimal,
    pub image_price: Option<Decimal>,
    pub reasoning_price: Option<Decimal>,
    pub cache_read_price: Option<Decimal>,
    pub request_price: Option<Decimal>,

    pub pricing_mode: PricingMode,

    pub avg_tokens_per_request: i32,

    pub effective_from: DateTime<Utc>,
    pub effective_until: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PricingMode {
    PerToken,
    PerRequest,
}

impl PricingMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PricingMode::PerToken => "per_token",
            PricingMode::PerRequest => "per_request",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "per_request" => PricingMode::PerRequest,
            _ => PricingMode::PerToken,
        }
    }
}

impl Default for PricingMode {
    fn default() -> Self {
        PricingMode::PerToken
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub prompt_cost: Decimal,
    pub completion_cost: Decimal,
    pub image_cost: Decimal,
    pub reasoning_cost: Decimal,
    pub cache_read_cost: Decimal,
    pub request_cost: Decimal,
    pub total: Decimal,
}

impl CostBreakdown {
    pub fn sum(&self) -> Decimal {
        self.prompt_cost
            + self.completion_cost
            + self.image_cost
            + self.reasoning_cost
            + self.cache_read_cost
            + self.request_cost
    }
}
