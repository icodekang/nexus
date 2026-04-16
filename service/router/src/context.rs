//! 路由上下文模块
//!
//! 定义路由请求时使用的上下文信息

use crate::RouteStrategy;

/// 路由请求的上下文
///
/// 包含选择提供商所需的所有信息
///
/// # 字段说明
/// - `model`: 目标模型标识符（如 "gpt-4o"）
/// - `strategy`: 使用的路由策略
/// - `provider_hint`: 可选的提供商提示（用于优先选择特定提供商）
#[derive(Debug, Clone)]
pub struct RouteContext {
    /// 目标模型标识符
    pub model: String,
    /// 路由策略
    pub strategy: RouteStrategy,
    /// 提供商提示（可选）
    pub provider_hint: Option<String>,
}

impl RouteContext {
    /// 创建新的路由上下文
    ///
    /// # 参数
    /// * `model` - 目标模型标识符
    pub fn new(model: String) -> Self {
        Self {
            model,
            strategy: RouteStrategy::default(),
            provider_hint: None,
        }
    }

    /// 设置路由策略
    ///
    /// # 参数
    /// * `strategy` - 路由策略
    pub fn with_strategy(mut self, strategy: RouteStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// 设置提供商提示
    ///
    /// 这是一个可选提示，用于优先选择特定提供商
    ///
    /// # 参数
    /// * `provider` - 提供商标识符
    pub fn with_provider_hint(mut self, provider: String) -> Self {
        self.provider_hint = Some(provider);
        self
    }
}
