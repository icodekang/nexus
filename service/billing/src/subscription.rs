use models::SubscriptionPlan;

/// Subscription plan details
pub struct PlanDetails {
    pub plan: SubscriptionPlan,
    pub name: &'static str,
    pub duration_days: i64,
    pub price_monthly: Option<f64>,
    pub price_yearly: Option<f64>,
    pub price_team: Option<f64>,
}

impl PlanDetails {
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

    pub fn get(plan: SubscriptionPlan) -> Option<Self> {
        Self::all().into_iter().find(|p| p.plan == plan)
    }
}

/// Subscription status info
pub struct SubscriptionStatusInfo {
    pub is_active: bool,
    pub days_remaining: i64,
    pub is_expiring_soon: bool,
    pub is_expired: bool,
}

impl SubscriptionStatusInfo {
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
