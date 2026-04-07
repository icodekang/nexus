use thiserror::Error;

#[derive(Error, Debug)]
pub enum BillingError {
    #[error("Invalid subscription plan: {0}")]
    InvalidPlan(String),

    #[error("Subscription not found")]
    SubscriptionNotFound,

    #[error("Subscription already expired")]
    SubscriptionExpired,

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Refund failed: {0}")]
    RefundFailed(String),
}
