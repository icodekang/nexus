//! Token 充值套餐

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPackage {
    pub id: Uuid,
    pub name: String,
    pub credits: Decimal,
    pub price: Decimal,
    pub currency: String,
    pub bonus_credits: Decimal,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}
