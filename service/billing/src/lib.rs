//! 计费模块
//!
//! 提供订阅管理和交易处理功能

pub mod subscription;
pub mod error;

pub use subscription::*;
pub use error::*;

use chrono::{DateTime, Utc};
use models::{User, Subscription, Transaction, SubscriptionPlan};
use models::subscription::{SubscriptionStatus, TransactionType, TransactionStatus};

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
