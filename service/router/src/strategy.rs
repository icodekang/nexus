//! 路由策略模块
//!
//! 定义了选择 AI 服务提供商时使用的各种策略

/// 路由策略枚举
///
/// 决定如何为请求选择最优的 AI 服务提供商
#[derive(Debug, Clone, Copy)]
pub enum RouteStrategy {
    /// 选择成本最低的提供商
    ///
    /// 根据提供商的优先级数值选择（数值越低越优先）
    /// 注：实际实现应使用真实定价数据
    Cheapest,

    /// 选择响应最快的提供商（最低延迟）
    ///
    /// 根据提供商的优先级数值作为延迟的代理指标
    /// 注：实际实现应使用真实延迟测量数据
    Fastest,

    /// 选择质量最高的提供商（最大上下文窗口）
    ///
    /// 根据提供商的优先级数值选择（数值越高越优先）
    /// 通常对应更大的模型上下文窗口
    Quality,

    /// 均衡评分（综合价格、延迟和质量）
    ///
    /// 使用优先级作为综合评分指标
    /// 注：实际实现应综合考虑价格、延迟和质量
    Balanced,
}

impl RouteStrategy {
    /// 从字符串解析路由策略
    ///
    /// # 参数
    /// * `s` - 策略名称字符串（不区分大小写）
    ///
    /// # 返回
    /// 对应的 RouteStrategy 枚举值，默认为 Balanced
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cheapest" => RouteStrategy::Cheapest,
            "fastest" => RouteStrategy::Fastest,
            "quality" => RouteStrategy::Quality,
            "balanced" => RouteStrategy::Balanced,
            _ => RouteStrategy::Balanced, // 默认策略
        }
    }
}

impl Default for RouteStrategy {
    fn default() -> Self {
        RouteStrategy::Balanced
    }
}
