//! 计费模块
//!
//! 提供订阅管理和交易处理功能

pub mod subscription;
pub mod error;

pub use subscription::*;
pub use error::*;

use chrono::Utc;
use models::{Subscription, Transaction, SubscriptionPlan};
use models::subscription::{SubscriptionStatus, TransactionType};

/// 计费服务
///
/// 提供订阅创建、交易处理和到期检查等功能
pub struct BillingService;

impl BillingService {
    /// 创建新的计费服务实例
    pub fn new() -> Self {
        Self
    }

    /// 创建新的订阅
    ///
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `plan` - 订阅计划
    /// * `duration_days` - 订阅时长（天数）
    ///
    /// # 返回
    /// 新创建的订阅记录
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

    /// 创建购买交易
    ///
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `plan` - 订阅计划
    /// * `amount` - 金额
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

    /// 创建续订交易
    ///
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `plan` - 订阅计划
    /// * `amount` - 金额
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

    /// 检查订阅是否即将到期
    ///
    /// # 参数
    /// * `subscription` - 订阅记录
    /// * `days` - 到期前天数阈值
    ///
    /// # 返回
    /// 如果订阅在指定天数内即将到期返回 true
    pub fn is_expiring_soon(&self, subscription: &Subscription, days: i64) -> bool {
        let threshold = Utc::now() + chrono::Duration::days(days);
        subscription.end_at <= threshold && subscription.end_at > Utc::now()
    }

    /// 计算距离到期的天数
    ///
    /// # 参数
    /// * `subscription` - 订阅记录
    ///
    /// # 返回
    /// 距离到期的天数（已到期返回 0）
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_create_subscription() {
        let billing = BillingService::new();
        let user_id = Uuid::new_v4();
        let sub = billing.create_subscription(user_id, SubscriptionPlan::Monthly, 30);

        assert_eq!(sub.user_id, user_id);
        assert_eq!(sub.plan, SubscriptionPlan::Monthly);
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert!(sub.end_at > sub.start_at);
        assert!(sub.end_at > Utc::now());
        assert!(sub.is_active());
    }

    #[test]
    fn test_subscription_expiry() {
        let billing = BillingService::new();
        let user_id = Uuid::new_v4();
        // Create a subscription that expires in 1 day
        let sub = billing.create_subscription(user_id, SubscriptionPlan::Monthly, 1);

        assert!(billing.is_expiring_soon(&sub, 2));  // expiring within 2 days
        assert!(billing.is_expiring_soon(&sub, 7));  // expiring within 7 days
        assert!(!sub.is_active());  // but note: is_active checks end_at > now, which may fail in edge cases
    }

    #[test]
    fn test_days_until_expiry() {
        let billing = BillingService::new();
        let user_id = Uuid::new_v4();
        let sub = billing.create_subscription(user_id, SubscriptionPlan::Yearly, 30);
        let days = billing.days_until_expiry(&sub);
        assert!(days >= 29 && days <= 30);
    }

    #[test]
    fn test_create_purchase_transaction() {
        let billing = BillingService::new();
        let user_id = Uuid::new_v4();
        let tx = billing.create_purchase_transaction(user_id, SubscriptionPlan::Monthly, 19.90);

        assert_eq!(tx.user_id, user_id);
        assert_eq!(tx.amount, 19.90);
        assert!(tx.plan == Some(SubscriptionPlan::Monthly));
        assert!(tx.description.unwrap().contains("Monthly"));
    }

    #[test]
    fn test_token_quota_values() {
        assert_eq!(SubscriptionPlan::None.monthly_token_quota(), 10_000);
        assert_eq!(SubscriptionPlan::ZeroToken.monthly_token_quota(), 100_000);
        assert_eq!(SubscriptionPlan::Monthly.monthly_token_quota(), 2_000_000);
        assert_eq!(SubscriptionPlan::Yearly.monthly_token_quota(), 2_000_000);
        assert_eq!(SubscriptionPlan::Team.monthly_token_quota(), 10_000_000);
        assert_eq!(SubscriptionPlan::Enterprise.monthly_token_quota(), i64::MAX);
    }
}
