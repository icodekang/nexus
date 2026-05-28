//! Token 消费记录

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenCharge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_log_id: Option<Uuid>,
    pub generation_id: Uuid,

    pub key_source: String,
    pub user_provider_key_id: Option<Uuid>,
    pub provider_key_id: Option<Uuid>,

    pub model_slug: String,
    pub provider_slug: String,

    pub input_tokens: i32,
    pub output_tokens: i32,
    pub reasoning_tokens: i32,
    pub image_count: i32,
    pub cache_read_tokens: i32,

    pub prompt_cost: Decimal,
    pub completion_cost: Decimal,
    pub image_cost: Decimal,
    pub reasoning_cost: Decimal,
    pub cache_read_cost: Decimal,
    pub request_cost: Decimal,
    pub total_cost: Decimal,

    pub is_free: bool,
    pub created_at: DateTime<Utc>,
}

impl TokenCharge {
    pub fn new(
        user_id: Uuid,
        generation_id: Uuid,
        model_slug: String,
        provider_slug: String,
        key_source: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            api_log_id: None,
            generation_id,
            key_source,
            user_provider_key_id: None,
            provider_key_id: None,
            model_slug,
            provider_slug,
            input_tokens: 0,
            output_tokens: 0,
            reasoning_tokens: 0,
            image_count: 0,
            cache_read_tokens: 0,
            prompt_cost: Decimal::ZERO,
            completion_cost: Decimal::ZERO,
            image_cost: Decimal::ZERO,
            reasoning_cost: Decimal::ZERO,
            cache_read_cost: Decimal::ZERO,
            request_cost: Decimal::ZERO,
            total_cost: Decimal::ZERO,
            is_free: false,
            created_at: now,
        }
    }
}
