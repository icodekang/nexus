//! 用户USD余额

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub user_id: Uuid,
    pub balance: Decimal,
    pub total_purchased: Decimal,
    pub total_consumed: Decimal,
    pub auto_topup_threshold: Option<Decimal>,
    pub auto_topup_amount: Option<Decimal>,
    pub updated_at: DateTime<Utc>,
}

impl UserBalance {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            balance: Decimal::ZERO,
            total_purchased: Decimal::ZERO,
            total_consumed: Decimal::ZERO,
            auto_topup_threshold: None,
            auto_topup_amount: None,
            updated_at: Utc::now(),
        }
    }
}
