//! 用户模块

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 用户账户
///
/// 包含用户的基本信息和订阅状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户 ID
    pub id: Uuid,
    /// 用户邮箱
    pub email: String,
    /// 手机号（可选）
    pub phone: Option<String>,
    /// 密码哈希（可选，用于邮箱密码登录）
    pub password_hash: Option<String>,
    /// 订阅计划
    pub subscription_plan: SubscriptionPlan,
    /// 订阅开始时间
    pub subscription_start: Option<DateTime<Utc>>,
    /// 订阅结束时间
    pub subscription_end: Option<DateTime<Utc>>,
    /// 是否为管理员
    pub is_admin: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// 创建新用户
    ///
    /// # 参数
    /// * `email` - 用户邮箱
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

    /// 设置密码哈希
    pub fn with_password(mut self, password_hash: String) -> Self {
        self.password_hash = Some(password_hash);
        self
    }

    /// 设置手机号
    pub fn with_phone(mut self, phone: String) -> Self {
        self.phone = Some(phone);
        self
    }

    /// 检查订阅是否活跃
    ///
    /// # 返回
    /// 如果订阅处于有效期内返回 true
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

/// 订阅计划枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionPlan {
    /// 无订阅
    None,
    /// 零 Token（10元/月）- 基于浏览器的免费访问
    ZeroToken,
    /// 月付
    Monthly,
    /// 年付
    Yearly,
    /// 团队版
    Team,
    /// 企业版
    Enterprise,
}

impl SubscriptionPlan {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionPlan::None => "none",
            SubscriptionPlan::ZeroToken => "zero_token",
            SubscriptionPlan::Monthly => "monthly",
            SubscriptionPlan::Yearly => "yearly",
            SubscriptionPlan::Team => "team",
            SubscriptionPlan::Enterprise => "enterprise",
        }
    }

    /// 从字符串解析
    ///
    /// # 参数
    /// * `s` - 字符串表示
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "zero_token" | "zerotoken" => SubscriptionPlan::ZeroToken,
            "monthly" => SubscriptionPlan::Monthly,
            "yearly" => SubscriptionPlan::Yearly,
            "team" => SubscriptionPlan::Team,
            "enterprise" => SubscriptionPlan::Enterprise,
            _ => SubscriptionPlan::None,
        }
    }

    /// 月度 Token 配额（输入 + 输出 Token 合计）
    ///
    /// # 返回
    /// 每月可用 Token 数量
    pub fn monthly_token_quota(&self) -> i64 {
        match self {
            SubscriptionPlan::None => 10_000,           // 免费: 10K tokens/月
            SubscriptionPlan::ZeroToken => 100_000,     // 10元/月: 100K tokens/月（基于浏览器）
            SubscriptionPlan::Monthly => 2_000_000,     // 19.9美元/月: 2M tokens/月
            SubscriptionPlan::Yearly => 2_000_000,      // 199美元/年: 2M tokens/月
            SubscriptionPlan::Team => 10_000_000,       // 99美元/月: 10M tokens/月
            SubscriptionPlan::Enterprise => i64::MAX,   // 企业版: 无限
        }
    }

    /// 是否支持自动续订
    pub fn supports_recurring(&self) -> bool {
        matches!(self, SubscriptionPlan::ZeroToken | SubscriptionPlan::Monthly | SubscriptionPlan::Team)
    }

    /// 一个计费周期的时长（天）
    pub fn billing_cycle_days(&self) -> i64 {
        match self {
            SubscriptionPlan::None => 30,
            SubscriptionPlan::ZeroToken => 30,
            SubscriptionPlan::Monthly => 30,
            SubscriptionPlan::Yearly => 365,
            SubscriptionPlan::Team => 30,
            SubscriptionPlan::Enterprise => 365,
        }
    }

    /// 是否使用基于浏览器的（零 Token）访问
    pub fn is_zero_token(&self) -> bool {
        matches!(self, SubscriptionPlan::ZeroToken)
    }
}

/// API Key
///
/// 用户生成的 API Key，用于认证和计费
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Key ID
    pub id: Uuid,
    /// 所属用户 ID
    pub user_id: Uuid,
    /// Key 的哈希值（存储用）
    pub key_hash: String,
    /// Key 的前缀（用于显示）
    pub key_prefix: String,
    /// Key 名称（可选）
    pub name: Option<String>,
    /// 是否激活
    pub is_active: bool,
    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl ApiKey {
    /// 创建新的 API Key
    ///
    /// # 参数
    /// * `user_id` - 所属用户 ID
    /// * `key_hash` - Key 的哈希值
    /// * `key_prefix` - Key 的前缀
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

    /// 设置 Key 名称
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}
