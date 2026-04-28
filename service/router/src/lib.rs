//! 路由器模块
//!
//! 提供提供商选择和路由功能的核心模块
//!
//! 主要功能：
//! - 根据策略选择最优的 AI 服务提供商
//! - 支持多种路由策略（成本、速度、质量、均衡）
//! - 维护提供商和模型列表

pub mod selector;
pub mod strategy;
pub mod context;
pub mod error;
pub mod key_scheduler;
pub mod session_health_checker;

pub use selector::*;
pub use strategy::*;
pub use context::*;
pub use error::*;
pub use key_scheduler::*;
pub use session_health_checker::*;

use models::{Provider, LlmModel, BuiltinModels, Providers};

/// 路由器核心
///
/// 负责根据模型和策略选择最优的 AI 服务提供商
///
/// # 字段说明
/// - `providers`: 所有可用的 AI 服务提供商
/// - `models`: 所有支持的 LLM 模型
pub struct RouterCore {
    providers: Vec<Provider>,
    models: Vec<LlmModel>,
}

impl RouterCore {
    /// 创建新的路由器实例
    ///
    /// 初始化时会加载所有内置提供商和模型
    pub fn new() -> Self {
        let providers = Providers::all();
        let models = BuiltinModels::all();
        
        Self { providers, models }
    }

    /// 根据模型选择最优的提供商
    ///
    /// # 参数
    /// * `model_slug` - 模型标识符（如 "gpt-4o"）
    /// * `strategy` - 路由策略
    ///
    /// # 返回
    /// 选中的提供商实例
    ///
    /// # 错误
    /// - `ModelNotFound`: 指定的模型不存在
    /// - `NoProviderAvailable`: 没有可用的提供商支持该模型
    pub async fn select_provider(
        &self,
        model_slug: &str,
        strategy: RouteStrategy,
    ) -> Result<Provider, RouterError> {
        // Find the model
        let model = self.models.iter()
            .find(|m| m.slug == model_slug)
            .ok_or(RouterError::ModelNotFound(model_slug.to_string()))?;

        // Find providers that support this model
        let candidates: Vec<&Provider> = self.providers.iter()
            .filter(|p| p.slug == model.provider_id && p.is_active)
            .collect();

        if candidates.is_empty() {
            return Err(RouterError::NoProviderAvailable);
        }

        // Select using the given strategy
        selector::select(&model, &candidates, strategy)
    }

    /// 获取所有可用的模型，按提供商分组
    ///
    /// # 返回
    /// HashMap，键为提供商标识符，值为该提供商支持的模型列表
    pub fn get_models_by_provider(&self) -> std::collections::HashMap<String, Vec<&LlmModel>> {
        let mut result: std::collections::HashMap<String, Vec<&LlmModel>> = std::collections::HashMap::new();
        
        for model in &self.models {
            result
                .entry(model.provider_id.clone())
                .or_default()
                .push(model);
        }
        
        result
    }

    /// 根据模型标识符获取模型信息
    ///
    /// # 参数
    /// * `slug` - 模型标识符
    ///
    /// # 返回
    /// 模型信息（如果存在）
    pub fn get_model(&self, slug: &str) -> Option<&LlmModel> {
        self.models.iter().find(|m| m.slug == slug)
    }
}

impl Default for RouterCore {
    fn default() -> Self {
        Self::new()
    }
}
