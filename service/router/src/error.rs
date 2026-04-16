//! 路由器错误模块
//!
//! 定义路由器操作过程中可能发生的各种错误类型

use thiserror::Error;

/// 路由器错误枚举
///
/// 包含路由选择过程中可能出现的各种错误情况
#[derive(Error, Debug)]
pub enum RouterError {
    /// 指定的模型不存在
    #[error("模型不存在: {0}")]
    ModelNotFound(String),

    /// 没有可用的提供商支持该模型
    #[error("没有可用的提供商支持此模型")]
    NoProviderAvailable,

    /// 指定的提供商不存在
    #[error("提供商不存在: {0}")]
    ProviderNotFound(String),

    /// 向提供商发送请求失败
    #[error("请求失败: {0}")]
    RequestFailed(String),

    /// 提供商返回了无效的响应
    #[error("无效的响应: {0}")]
    InvalidResponse(String),
}
