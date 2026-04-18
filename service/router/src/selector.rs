//! 提供商选择器模块
//!
//! 负责根据路由策略从多个提供商中选择最优的一个

use models::{Provider, LlmModel};

use crate::{RouteStrategy, RouterError};

/// 根据策略选择最优的提供商
///
/// # 参数
/// * `model` - 目标 LLM 模型
/// * `providers` - 可用的提供商列表
/// * `strategy` - 路由策略
///
/// # 返回
/// 选中的提供商实例
///
/// # 策略说明
/// - `Cheapest`: 选择优先级数值最低的提供商
/// - `Fastest`: 选择优先级数值最低的提供商（作为延迟的代理指标）
/// - `Quality`: 选择优先级数值最高的提供商
/// - `Balanced`: 选择优先级数值最低的提供商
///
/// # 注意
/// 实际生产环境中，Cheapest 和 Fastest 策略应使用真实的定价和延迟数据
pub fn select(
    _model: &LlmModel,
    providers: &[&Provider],
    strategy: RouteStrategy,
) -> Result<Provider, RouterError> {
    if providers.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    // 过滤出活跃的提供商
    let active: Vec<&Provider> = providers.iter()
        .filter(|p| p.is_active)
        .cloned()
        .collect();

    if active.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    match strategy {
        RouteStrategy::Cheapest => {
            // 按优先级排序（数值越低越优先 = 越便宜）
            // 注：实际系统应使用真实定价数据
            let mut sorted = active.clone();
            sorted.sort_by_key(|p| p.priority);
            Ok((*sorted.first().unwrap()).clone())
        }
        RouteStrategy::Fastest => {
            // 按优先级排序作为延迟的代理指标
            // 注：实际系统应使用真实延迟测量数据
            let mut sorted = active.clone();
            sorted.sort_by_key(|p| p.priority);
            Ok((*sorted.first().unwrap()).clone())
        }
        RouteStrategy::Quality => {
            // 优先级数值越高 = 质量越好（对应更大的上下文窗口）
            let mut sorted = active.clone();
            sorted.sort_by(|a, b| {
                // 优先级数值越高，质量越好
                b.priority.cmp(&a.priority)
            });
            Ok((*sorted.first().unwrap()).clone())
        }
        RouteStrategy::Balanced => {
            // 均衡模式使用优先级作为综合评分
            // 注：实际系统应综合价格、延迟和质量
            let mut sorted = active.clone();
            sorted.sort_by_key(|p| p.priority);
            Ok((*sorted.first().unwrap()).clone())
        }
    }
}

/// 选择提供商并返回按优先级排序的列表（用于故障转移）
///
/// 当主提供商请求失败时，可以按顺序尝试列表中的其他提供商
///
/// # 参数
/// * `model` - 目标 LLM 模型
/// * `providers` - 可用的提供商列表
/// * `strategy` - 路由策略
///
/// # 返回
/// 按优先级排序的活跃提供商列表
pub fn select_with_fallback(
    _model: &LlmModel,
    providers: &[&Provider],
    strategy: RouteStrategy,
) -> Result<Vec<Provider>, RouterError> {
    if providers.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    // 过滤出活跃的提供商
    let mut active: Vec<Provider> = providers.iter()
        .filter(|p| p.is_active)
        .cloned()
        .cloned()
        .collect();

    if active.is_empty() {
        return Err(RouterError::NoProviderAvailable);
    }

    match strategy {
        RouteStrategy::Cheapest | RouteStrategy::Fastest | RouteStrategy::Balanced => {
            active.sort_by_key(|p| p.priority);
        }
        RouteStrategy::Quality => {
            active.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
    }

    Ok(active)
}
