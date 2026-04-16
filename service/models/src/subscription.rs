//! 订阅模块

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::user::SubscriptionPlan;

/// 订阅记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// 订阅 ID
    pub id: Uuid,
    /// 用户 ID
    pub user_id: Uuid,
    /// 订阅计划
    pub plan: SubscriptionPlan,
    /// 订阅状态
    pub status: SubscriptionStatus,
    /// 开始时间
    pub start_at: DateTime<Utc>,
    /// 结束时间
    pub end_at: DateTime<Utc>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Subscription {
    /// 创建新的订阅
    ///
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `plan` - 订阅计划
    /// * `duration_days` - 订阅时长（天）
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

    /// 检查订阅是否活跃
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        self.status == SubscriptionStatus::Active && now <= self.end_at
    }
}

/// 订阅状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    /// 活跃
    Active,
    /// 已过期
    Expired,
    /// 已取消
    Cancelled,
}

/// 订阅计划信息（用于价格展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlanInfo {
    /// 订阅计划
    pub plan: SubscriptionPlan,
    /// 计划名称
    pub name: String,
    /// 月付价格
    pub price_monthly: f64,
    /// 年付价格
    pub price_yearly: f64,
    /// 团队月付价格
    pub price_team_monthly: f64,
    /// 功能列表
    pub features: Vec<String>,
}

impl SubscriptionPlanInfo {
    /// 获取所有订阅计划信息
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                plan: SubscriptionPlan::ZeroToken,
                name: "零Token".to_string(),
                price_monthly: 10.0,
                price_yearly: 0.0,
                price_team_monthly: 0.0,
                features: vec![
                    "浏览器模拟访问大模型".to_string(),
                    "无需 API Key".to_string(),
                    "10万 tokens/月".to_string(),
                    "支持 Claude.ai".to_string(),
                    "支持 ChatGPT".to_string(),
                ],
            },
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

/// 订阅交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// 交易 ID
    pub id: Uuid,
    /// 用户 ID
    pub user_id: Uuid,
    /// 交易类型
    pub transaction_type: TransactionType,
    /// 金额（正数表示支付，负数表示退款）
    pub amount: f64,
    /// 订阅计划（可选）
    pub plan: Option<SubscriptionPlan>,
    /// 交易状态
    pub status: TransactionStatus,
    /// 描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    /// 创建新的交易
    ///
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `transaction_type` - 交易类型
    /// * `amount` - 金额
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

    /// 设置订阅计划
    pub fn with_plan(mut self, plan: SubscriptionPlan) -> Self {
        self.plan = Some(plan);
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// 交易类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// 订阅购买
    SubscriptionPurchase,
    /// 订阅续订
    SubscriptionRenewal,
    /// 订阅取消
    SubscriptionCancellation,
    /// 退款
    Refund,
}

/// 交易状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    /// 待处理
    Pending,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已退款
    Refunded,
}
