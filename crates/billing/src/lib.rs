pub mod credits;
pub mod pricing;
pub mod deduction;
pub mod invoice;

use rust_decimal::Decimal;
use models::usage::{UsageLog, Transaction, TransactionType};
use thiserror::Error;

pub struct BillingService;

impl BillingService {
    pub fn new() -> Self {
        Self
    }

    /// Calculate cost for token usage
    pub fn calculate_cost(
        &self,
        input_tokens: i32,
        output_tokens: i32,
        price_input: Decimal,
        price_output: Decimal,
    ) -> Decimal {
        let input_cost = price_input * Decimal::from(input_tokens) / Decimal::new(1_000_000, 0);
        let output_cost = price_output * Decimal::from(output_tokens) / Decimal::new(1_000_000, 0);
        input_cost + output_cost
    }

    /// Deduct credits from user
    pub async fn deduct_credits(
        &self,
        user_id: uuid::Uuid,
        amount: Decimal,
    ) -> Result<Transaction, BillingError> {
        // TODO: Use database transaction
        if amount <= Decimal::ZERO {
            return Err(BillingError::InvalidAmount);
        }

        let transaction = Transaction {
            id: uuid::Uuid::new_v4(),
            user_id,
            transaction_type: TransactionType::UsageDeduction,
            amount: -amount,
            balance_before: Decimal::new(10000, 4), // TODO: get from DB
            balance_after: Decimal::new(9990, 4),   // TODO: calculate
            description: Some("Usage deduction".to_string()),
            created_at: chrono::Utc::now(),
        };

        Ok(transaction)
    }

    /// Add credits to user (purchase or refund)
    pub async fn add_credits(
        &self,
        user_id: uuid::Uuid,
        amount: Decimal,
        description: &str,
    ) -> Result<Transaction, BillingError> {
        if amount <= Decimal::ZERO {
            return Err(BillingError::InvalidAmount);
        }

        let transaction = Transaction {
            id: uuid::Uuid::new_v4(),
            user_id,
            transaction_type: TransactionType::CreditsPurchase,
            amount,
            balance_before: Decimal::ZERO,
            balance_after: amount,
            description: Some(description.to_string()),
            created_at: chrono::Utc::now(),
        };

        Ok(transaction)
    }
}

impl Default for BillingService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum BillingError {
    #[error("Insufficient credits")]
    InsufficientCredits,

    #[error("Invalid amount")]
    InvalidAmount,

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}
