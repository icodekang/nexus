use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::user::SubscriptionPlan;

/// Subscription record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan: SubscriptionPlan,
    pub status: SubscriptionStatus,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Subscription {
    pub fn new(user_id: Uuid, plan: SubscriptionPlan, duration_days: i64) -> Self {
        let now = Utc::now();
        let end_at = now + chrono::Duration::days(duration_days);
        
        Self {
            id: Uuid::new_v4(),
            user_id,
            plan,
            status: SubscriptionStatus::Active,
            start_at: now,
            end_at,
            created_at: now,
        }
    }

    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        self.status == SubscriptionStatus::Active && now <= self.end_at
    }
}

/// Subscription status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Active,
    Expired,
    Cancelled,
}

/// Subscription plan info (for pricing display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlanInfo {
    pub plan: SubscriptionPlan,
    pub name: String,
    pub price_monthly: f64,
    pub price_yearly: f64,
    pub price_team_monthly: f64,
    pub features: Vec<String>,
}

impl SubscriptionPlanInfo {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                plan: SubscriptionPlan::Monthly,
                name: "月付".to_string(),
                price_monthly: 19.9,
                price_yearly: 0.0,
                price_team_monthly: 0.0,
                features: vec![
                    "无限使用所有模型".to_string(),
                    "访问 OpenAI 模型".to_string(),
                    "访问 Anthropic 模型".to_string(),
                    "访问 Google 模型".to_string(),
                    "访问 DeepSeek 模型".to_string(),
                ],
            },
            Self {
                plan: SubscriptionPlan::Yearly,
                name: "年付".to_string(),
                price_monthly: 0.0,
                price_yearly: 199.0,
                price_team_monthly: 0.0,
                features: vec![
                    "所有月付功能".to_string(),
                    "节省 17%".to_string(),
                    "优先客服支持".to_string(),
                ],
            },
            Self {
                plan: SubscriptionPlan::Team,
                name: "团队版".to_string(),
                price_monthly: 99.0,
                price_yearly: 0.0,
                price_team_monthly: 99.0,
                features: vec![
                    "5 个席位".to_string(),
                    "所有模型访问".to_string(),
                    "团队管理后台".to_string(),
                    "优先客服支持".to_string(),
                ],
            },
            Self {
                plan: SubscriptionPlan::Enterprise,
                name: "企业版".to_string(),
                price_monthly: 0.0,
                price_yearly: 0.0,
                price_team_monthly: 0.0,
                features: vec![
                    "无限席位".to_string(),
                    "私有部署选项".to_string(),
                    "专属客户经理".to_string(),
                    "SLA 保障".to_string(),
                    "定制集成".to_string(),
                ],
            },
        ]
    }
}

/// Transaction record for subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: f64,  // Positive for payments, negative for refunds
    pub plan: Option<SubscriptionPlan>,
    pub status: TransactionStatus,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    pub fn new(user_id: Uuid, transaction_type: TransactionType, amount: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            transaction_type,
            amount,
            plan: None,
            status: TransactionStatus::Completed,
            description: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_plan(mut self, plan: SubscriptionPlan) -> Self {
        self.plan = Some(plan);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    SubscriptionPurchase,
    SubscriptionRenewal,
    SubscriptionCancellation,
    Refund,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}
