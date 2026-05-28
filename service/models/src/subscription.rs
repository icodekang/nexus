//! 交易模块

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// 交易 ID
    pub id: Uuid,
    /// 用户 ID
    pub user_id: Uuid,
    /// 交易类型
    pub transaction_type: TransactionType,
    /// 金额
    pub amount: f64,
    /// 关联模型/套餐（可选）
    pub plan: Option<String>,
    /// 交易状态
    pub status: TransactionStatus,
    /// 描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    /// 创建新的交易
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

    pub fn with_plan(mut self, plan: String) -> Self {
        self.plan = Some(plan);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// 交易类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// 购买 Token 套餐
    TokenPurchase,
    /// 退款
    Refund,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::TokenPurchase => "token_purchase",
            TransactionType::Refund => "refund",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "refund" => TransactionType::Refund,
            _ => TransactionType::TokenPurchase,
        }
    }
}

/// 交易状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Completed => "completed",
            TransactionStatus::Failed => "failed",
            TransactionStatus::Refunded => "refunded",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => TransactionStatus::Pending,
            "failed" => TransactionStatus::Failed,
            "refunded" => TransactionStatus::Refunded,
            _ => TransactionStatus::Completed,
        }
    }
}
