//! 计费错误模块

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BillingError {
    #[error("余额不足")]
    InsufficientBalance,

    #[error("无效的定价模式: {0}")]
    InvalidPricingMode(String),

    #[error("未找到模型定价: {0}")]
    PricingNotFound(String),

    #[error("计费失败: {0}")]
    ChargeFailed(String),

    #[error("支付失败: {0}")]
    PaymentFailed(String),

    #[error("退款失败: {0}")]
    RefundFailed(String),
}
