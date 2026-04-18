//! 订阅模型单元测试
//!
//! 测试 Subscription 和 Transaction 结构的创建和状态转换

use nexus_models::subscription::{
    Subscription, SubscriptionPlan, SubscriptionStatus,
    Transaction, TransactionType, TransactionStatus,
};
use chrono::{Utc, Duration};

#[test]
fn test_subscription_creation() {
    let now = Utc::now();
    let subscription = Subscription::new(
        SubscriptionPlan::Monthly,
        now,
        now + Duration::days(30),
    );

    assert_eq!(subscription.plan, SubscriptionPlan::Monthly);
    assert_eq!(subscription.status, SubscriptionStatus::Active);
    assert!(subscription.is_active());
}

#[test]
fn test_subscription_status_transitions() {
    let now = Utc::now();
    let mut subscription = Subscription::new(
        SubscriptionPlan::Monthly,
        now,
        now + Duration::days(30),
    );

    // Active -> Expired
    subscription.status = SubscriptionStatus::Expired;
    assert!(!subscription.is_active());
    assert!(subscription.is_expired());

    // Expired -> Active (续费)
    subscription.status = SubscriptionStatus::Active;
    assert!(subscription.is_active());
}

#[test]
fn test_subscription_cancel() {
    let now = Utc::now();
    let mut subscription = Subscription::new(
        SubscriptionPlan::Monthly,
        now,
        now + Duration::days(30),
    );

    subscription.cancel();
    assert_eq!(subscription.status, SubscriptionStatus::Canceled);
}

#[test]
fn test_transaction_creation() {
    let transaction = Transaction::new(
        UserId::new(),
        TransactionType::Purchase,
        Decimal::new(1990, 2), // $19.90
        Some(SubscriptionPlan::Monthly),
    );

    assert_eq!(transaction.transaction_type, TransactionType::Purchase);
    assert_eq!(transaction.amount, Decimal::new(1990, 2));
    assert_eq!(transaction.status, TransactionStatus::Completed);
}

#[test]
fn test_transaction_refund() {
    let mut transaction = Transaction::new(
        UserId::new(),
        TransactionType::Purchase,
        Decimal::new(1990, 2),
        Some(SubscriptionPlan::Monthly),
    );

    transaction.refund();
    assert_eq!(transaction.status, TransactionStatus::Refunded);
}

#[test]
fn test_transaction_types() {
    assert_eq!(TransactionType::Purchase.as_str(), "purchase");
    assert_eq!(TransactionType::Renewal.as_str(), "renewal");
    assert_eq!(TransactionType::Refund.as_str(), "refund");
}

#[test]
fn test_transaction_status() {
    assert_eq!(TransactionStatus::Pending.as_str(), "pending");
    assert_eq!(TransactionStatus::Completed.as_str(), "completed");
    assert_eq!(TransactionStatus::Refunded.as_str(), "refunded");
    assert_eq!(TransactionStatus::Failed.as_str(), "failed");
}

#[test]
fn test_subscription_renewal() {
    let now = Utc::now();
    let old_end = now;
    let new_end = now + Duration::days(30);

    let mut subscription = Subscription::new(
        SubscriptionPlan::Monthly,
        now - Duration::days(30),
        old_end,
    );

    subscription.renew(new_end);
    assert_eq!(subscription.subscription_end, new_end);
    assert_eq!(subscription.status, SubscriptionStatus::Active);
}

#[test]
fn test_free_plan() {
    let now = Utc::now();
    let subscription = Subscription::new(
        SubscriptionPlan::None,
        now,
        now + Duration::days(365),
    );

    assert!(!subscription.is_active());
    assert!(subscription.is_expired()); // 无订阅视为过期
}
