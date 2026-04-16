//! 计费错误模块

use thiserror::Error;

/// 计费错误枚举
///
/// 包含计费过程中可能出现的各种错误情况
#[derive(Error, Debug)]
pub enum BillingError {
    /// 无效的订阅计划
    #[error("无效的订阅计划: {0}")]
    InvalidPlan(String),

    /// 订阅不存在
    #[error("订阅不存在")]
    SubscriptionNotFound,

    /// 订阅已过期
    #[error("订阅已过期")]
    SubscriptionExpired,

    /// 支付失败
    #[error("支付失败: {0}")]
    PaymentFailed(String),

    /// 退款失败
    #[error("退款失败: {0}")]
    RefundFailed(String),
}
