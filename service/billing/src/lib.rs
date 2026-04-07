pub mod subscription;
pub mod error;

pub use subscription::*;
pub use error::*;

use chrono::{DateTime, Utc};
use models::{User, Subscription, Transaction, subscription::{SubscriptionPlan, SubscriptionStatus, TransactionType, TransactionStatus}};

pub struct BillingService;

impl BillingService {
    pub fn new() -> Self {
        Self
    }

    /// Create a new subscription
    pub fn create_subscription(
        &self,
        user_id: uuid::Uuid,
        plan: SubscriptionPlan,
        duration_days: i64,
    ) -> Subscription {
        let now = Utc::now();
        let end = now + chrono::Duration::days(duration_days);

        Subscription {
            id: uuid::Uuid::new_v4(),
            user_id,
            plan,
            status: SubscriptionStatus::Active,
            start_at: now,
            end_at: end,
            created_at: now,
        }
    }

    /// Create a purchase transaction
    pub fn create_purchase_transaction(
        &self,
        user_id: uuid::Uuid,
        plan: SubscriptionPlan,
        amount: f64,
    ) -> Transaction {
        Transaction::new(user_id, TransactionType::SubscriptionPurchase, amount)
            .with_plan(plan)
            .with_description(format!("Purchase of {:?} plan", plan))
    }

    /// Create a renewal transaction
    pub fn create_renewal_transaction(
        &self,
        user_id: uuid::Uuid,
        plan: SubscriptionPlan,
        amount: f64,
    ) -> Transaction {
        Transaction::new(user_id, TransactionType::SubscriptionRenewal, amount)
            .with_plan(plan)
            .with_description(format!("Renewal of {:?} plan", plan))
    }

    /// Check if a subscription is about to expire (within days)
    pub fn is_expiring_soon(&self, subscription: &Subscription, days: i64) -> bool {
        let threshold = Utc::now() + chrono::Duration::days(days);
        subscription.end_at <= threshold && subscription.end_at > Utc::now()
    }

    /// Calculate days until expiration
    pub fn days_until_expiry(&self, subscription: &Subscription) -> i64 {
        let now = Utc::now();
        if subscription.end_at > now {
            (subscription.end_at - now).num_days()
        } else {
            0
        }
    }
}

impl Default for BillingService {
    fn default() -> Self {
        Self::new()
    }
}
