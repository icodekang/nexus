use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Provider;

/// Supported LLM models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModel {
    pub id: Uuid,
    pub provider_id: String,
    pub name: String,
    pub slug: String,
    pub model_id: String,  // Provider's actual model ID
    pub mode: ModelMode,
    pub context_window: i32,
    pub capabilities: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl LlmModel {
    pub fn new(
        provider_id: String,
        name: String,
        slug: String,
        model_id: String,
        mode: ModelMode,
        context_window: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider_id,
            name,
            slug,
            model_id,
            mode,
            context_window,
            capabilities: vec![],
            is_active: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }
}

/// Model mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelMode {
    Chat,
    Completion,
    Embedding,
}

impl ModelMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelMode::Chat => "chat",
            ModelMode::Completion => "completion",
            ModelMode::Embedding => "embedding",
        }
    }
}

/// Model with provider info (for API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelWithProvider {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub provider_name: String,
    pub context_window: i32,
    pub capabilities: Vec<String>,
}

impl ModelWithProvider {
    pub fn from_model(model: &LlmModel, provider: &Provider) -> Self {
        Self {
            id: model.slug.clone(),
            name: model.name.clone(),
            provider: provider.slug.clone(),
            provider_name: provider.name.clone(),
            context_window: model.context_window,
            capabilities: model.capabilities.clone(),
        }
    }
}

/// Built-in models
pub struct BuiltinModels;

impl BuiltinModels {
    pub fn all() -> Vec<LlmModel> {
        use ModelMode::*;
        
        vec![
            // OpenAI models
            Self::chat("openai", "GPT-4o", "gpt-4o", "gpt-4o", 128000, vec!["vision".to_string(), "function_call".to_string()]),
            Self::chat("openai", "GPT-4o Mini", "gpt-4o-mini", "gpt-4o-mini", 128000, vec!["function_call".to_string()]),
            Self::chat("openai", "GPT-4 Turbo", "gpt-4-turbo", "gpt-4-turbo", 128000, vec!["vision".to_string()]),
            Self::chat("openai", "GPT-3.5 Turbo", "gpt-3.5-turbo", "gpt-3.5-turbo-1106", 16385, vec![]),
            
            // Anthropic models
            Self::chat("anthropic", "Claude 3.5 Sonnet", "claude-3-5-sonnet", "claude-3-5-sonnet-20241022", 200000, vec!["vision".to_string()]),
            Self::chat("anthropic", "Claude 3 Opus", "claude-3-opus", "claude-3-opus-20240229", 200000, vec!["vision".to_string()]),
            Self::chat("anthropic", "Claude 3 Haiku", "claude-3-haiku", "claude-3-haiku-20240307", 200000, vec![]),
            
            // Google models
            Self::chat("google", "Gemini 1.5 Pro", "gemini-1-5-pro", "gemini-1.5-pro", 2000000, vec!["vision".to_string()]),
            Self::chat("google", "Gemini 1.5 Flash", "gemini-1-5-flash", "gemini-1.5-flash", 1000000, vec!["vision".to_string()]),
            Self::chat("google", "Gemini 1.0 Pro", "gemini-1-0-pro", "gemini-pro", 32768, vec![]),
            
            // DeepSeek models
            Self::chat("deepseek", "DeepSeek V3", "deepseek-chat", "deepseek-chat", 64000, vec![]),
            Self::chat("deepseek", "DeepSeek Coder", "deepseek-coder", "deepseek-coder", 64000, vec![]),
        ]
    }
    
    fn chat(provider: &str, name: &str, slug: &str, model_id: &str, context_window: i32, capabilities: Vec<String>) -> LlmModel {
        LlmModel::new(
            provider.to_string(),
            name.to_string(),
            slug.to_string(),
            model_id.to_string(),
            ModelMode::Chat,
            context_window,
        )
        .with_capabilities(capabilities)
    }
}
