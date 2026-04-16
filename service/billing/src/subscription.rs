//! 订阅计划详情模块

use models::SubscriptionPlan;

/// 订阅计划详情
///
/// 包含订阅计划的价格和时长信息
pub struct PlanDetails {
    /// 订阅计划类型
    pub plan: SubscriptionPlan,
    /// 计划名称
    pub name: &'static str,
    /// 订阅时长（天数）
    pub duration_days: i64,
    /// 月付价格
    pub price_monthly: Option<f64>,
    /// 年付价格
    pub price_yearly: Option<f64>,
    /// 团队价格
    pub price_team: Option<f64>,
}

impl PlanDetails {
    /// 获取所有订阅计划
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                plan: SubscriptionPlan::Monthly,
                name: "Monthly",
                duration_days: 30,
                price_monthly: Some(19.9),
                price_yearly: None,
                price_team: None,
            },
            Self {
                plan: SubscriptionPlan::Yearly,
                name: "Yearly",
                duration_days: 365,
                price_monthly: None,
                price_yearly: Some(199.0),
                price_team: None,
            },
            Self {
                plan: SubscriptionPlan::Team,
                name: "Team",
                duration_days: 30,
                price_monthly: Some(99.0),
                price_yearly: None,
                price_team: Some(99.0),
            },
            Self {
                plan: SubscriptionPlan::Enterprise,
                name: "Enterprise",
                duration_days: 365,
                price_monthly: None,
                price_yearly: None,
                price_team: None,
            },
        ]
    }

    /// 获取指定计划详情
    ///
    /// # 参数
    /// * `plan` - 订阅计划类型
    pub fn get(plan: SubscriptionPlan) -> Option<Self> {
        Self::all().into_iter().find(|p| p.plan == plan)
    }
}

/// 订阅状态信息
///
/// 包含订阅的当前状态和剩余天数等详细信息
pub struct SubscriptionStatusInfo {
    /// 是否活跃
    pub is_active: bool,
    /// 剩余天数
    pub days_remaining: i64,
    /// 是否即将到期（7天内）
    pub is_expiring_soon: bool,
    /// 是否已过期
    pub is_expired: bool,
}

impl SubscriptionStatusInfo {
    /// 从订阅记录创建状态信息
    ///
    /// # 参数
    /// * `subscription` - 订阅记录
    pub fn from_subscription(subscription: &models::Subscription) -> Self {
        use chrono::Utc;

        let now = Utc::now();
        let is_expired = subscription.end_at <= now;
        let is_active = subscription.status == models::subscription::SubscriptionStatus::Active && !is_expired;
        let days_remaining = if is_expired {
            0
        } else {
            (subscription.end_at - now).num_days().max(0)
        };
        let is_expiring_soon = days_remaining <= 7 && days_remaining > 0;

        Self {
            is_active,
            days_remaining,
            is_expiring_soon,
            is_expired,
        }
    }
}
