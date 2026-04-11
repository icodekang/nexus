use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    pub password_hash: Option<String>,
    pub subscription_plan: SubscriptionPlan,
    pub subscription_start: Option<DateTime<Utc>>,
    pub subscription_end: Option<DateTime<Utc>>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            phone: None,
            password_hash: None,
            subscription_plan: SubscriptionPlan::None,
            subscription_start: None,
            subscription_end: None,
            is_admin: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_password(mut self, password_hash: String) -> Self {
        self.password_hash = Some(password_hash);
        self
    }

    pub fn with_phone(mut self, phone: String) -> Self {
        self.phone = Some(phone);
        self
    }

    pub fn is_subscription_active(&self) -> bool {
        if let (Some(start), Some(end), SubscriptionPlan::None) = 
            (self.subscription_start, self.subscription_end, &self.subscription_plan) 
        {
            return false;
        }
        
        match (&self.subscription_start, &self.subscription_end, &self.subscription_plan) {
            (Some(start), Some(end), plan) if *plan != SubscriptionPlan::None => {
                let now = Utc::now();
                now >= *start && now <= *end
            }
            _ => false,
        }
    }
}

/// Subscription plan
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionPlan {
    None,
    Monthly,
    Yearly,
    Team,
    Enterprise,
}

impl SubscriptionPlan {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionPlan::None => "none",
            SubscriptionPlan::Monthly => "monthly",
            SubscriptionPlan::Yearly => "yearly",
            SubscriptionPlan::Team => "team",
            SubscriptionPlan::Enterprise => "enterprise",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "monthly" => SubscriptionPlan::Monthly,
            "yearly" => SubscriptionPlan::Yearly,
            "team" => SubscriptionPlan::Team,
            "enterprise" => SubscriptionPlan::Enterprise,
            _ => SubscriptionPlan::None,
        }
    }

    /// Monthly token quota for this plan (input + output tokens combined)
    pub fn monthly_token_quota(&self) -> i64 {
        match self {
            SubscriptionPlan::None => 10_000,           // Free: 10K tokens/month
            SubscriptionPlan::Monthly => 2_000_000,     // $19.9/mo: 2M tokens/month
            SubscriptionPlan::Yearly => 2_000_000,      // $199/yr: 2M tokens/month
            SubscriptionPlan::Team => 10_000_000,       // $99/mo: 10M tokens/month
            SubscriptionPlan::Enterprise => i64::MAX,   // Enterprise: unlimited
        }
    }

    /// Whether this plan supports auto-renewal
    pub fn supports_recurring(&self) -> bool {
        matches!(self, SubscriptionPlan::Monthly | SubscriptionPlan::Team)
    }

    /// Duration in days for one billing cycle
    pub fn billing_cycle_days(&self) -> i64 {
        match self {
            SubscriptionPlan::None => 30,
            SubscriptionPlan::Monthly => 30,
            SubscriptionPlan::Yearly => 365,
            SubscriptionPlan::Team => 30,
            SubscriptionPlan::Enterprise => 365,
        }
    }
}

/// API Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,
    pub key_prefix: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ApiKey {
    pub fn new(user_id: Uuid, key_hash: String, key_prefix: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            key_hash,
            key_prefix,
            name: None,
            is_active: true,
            last_used_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}
