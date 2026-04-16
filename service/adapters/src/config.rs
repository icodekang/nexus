//! 提供商配置模块
//!
//! 定义了如何连接各个 LLM 提供商的配置

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 提供商 API 配置
///
/// 包含连接提供商所需的所有配置信息
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// 提供商标识符（如 openai, anthropic, google, deepseek 或自定义）
    pub id: String,
    /// 人类可读的名称
    pub name: String,
    /// API 端点的基础 URL
    pub base_url: String,
    /// 认证方式
    pub auth: AuthConfig,
    /// Chat completions 端点路径
    pub chat_path: String,
    /// Embeddings 端点路径
    pub embeddings_path: String,
    /// 流式响应端点路径（如果与 chat 不同）
    pub stream_path: Option<String>,
    /// 默认请求头（如 "anthropic-version"）
    pub headers: HashMap<String, String>,
    /// 是否使用 OpenAI 兼容的响应格式
    pub openai_compatible: bool,
    /// 是否为自定义提供商（用户定义）
    pub is_custom: bool,
}

/// 认证配置枚举
///
/// 定义了与提供商 API 认证的方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    /// Bearer Token (Authorization: Bearer <token>)
    /// 用于 OpenAI、DeepSeek 等
    Bearer,
    /// API Key 在请求头中 (x-api-key: <token>)
    /// 用于 Anthropic 等
    ApiKey,
    /// 查询参数 (?key=<token>)
    /// 用于 Google 等
    QueryKey,
}

impl ProviderConfig {
    pub fn new(id: &str, name: &str, base_url: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            auth: AuthConfig::Bearer,
            chat_path: "/chat/completions".to_string(),
            embeddings_path: "/embeddings".to_string(),
            stream_path: None,
            headers: HashMap::new(),
            openai_compatible: true,
            is_custom: false,
        }
    }

    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    pub fn with_headers(mut self, headers: HashMap<&str, &str>) -> Self {
        self.headers = headers
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self
    }

    pub fn with_chat_path(mut self, path: &str) -> Self {
        self.chat_path = path.to_string();
        self
    }

    pub fn with_embeddings_path(mut self, path: &str) -> Self {
        self.embeddings_path = path.to_string();
        self
    }

    pub fn with_stream_path(mut self, path: &str) -> Self {
        self.stream_path = Some(path.to_string());
        self
    }

    pub fn with_openai_compatible(mut self, compatible: bool) -> Self {
        self.openai_compatible = compatible;
        self
    }
}

/// Built-in provider configurations
pub struct BuiltinProviders;

impl BuiltinProviders {
    pub fn all() -> Vec<ProviderConfig> {
        vec![
            Self::openai(),
            Self::anthropic(),
            Self::google(),
            Self::deepseek(),
        ]
    }

    pub fn get(id: &str) -> Option<ProviderConfig> {
        match id {
            "openai" => Some(Self::openai()),
            "anthropic" => Some(Self::anthropic()),
            "google" => Some(Self::google()),
            "deepseek" => Some(Self::deepseek()),
            _ => None,
        }
    }

    pub fn openai() -> ProviderConfig {
        ProviderConfig::new("openai", "OpenAI", "https://api.openai.com/v1")
            .with_auth(AuthConfig::Bearer)
    }

    pub fn anthropic() -> ProviderConfig {
        let mut headers = HashMap::new();
        headers.insert("anthropic-version", "2023-06-01");

        ProviderConfig::new("anthropic", "Anthropic", "https://api.anthropic.com/v1")
            .with_auth(AuthConfig::ApiKey)
            .with_headers(headers)
            .with_openai_compatible(false)
    }

    pub fn google() -> ProviderConfig {
        ProviderConfig::new("google", "Google", "https://generativelanguage.googleapis.com/v1beta")
            .with_auth(AuthConfig::QueryKey)
            .with_openai_compatible(false)
    }

    pub fn deepseek() -> ProviderConfig {
        ProviderConfig::new("deepseek", "DeepSeek", "https://api.deepseek.com/v1")
            .with_auth(AuthConfig::Bearer)
    }
}

/// Custom provider loader from environment
/// Format: CUSTOM_PROVIDERS=[{"id":"ollama","name":"Ollama","base_url":"http://localhost:11434","auth":"bearer","api_key_env":"OLLAMA_API_KEY"}]
pub struct CustomProviders;

impl CustomProviders {
    /// Load custom providers from CUSTOM_PROVIDERS env var (JSON array)
    pub fn load_from_env() -> Vec<ProviderConfig> {
        let Ok(json_str) = std::env::var("CUSTOM_PROVIDERS") else {
            return Vec::new();
        };

        let Ok(providers) = serde_json::from_str::<Vec<CustomProviderDef>>(&json_str) else {
            tracing::warn!("Failed to parse CUSTOM_PROVIDERS env var");
            return Vec::new();
        };

        providers
            .into_iter()
            .map(|def| {
                let auth = match def.auth.to_lowercase().as_str() {
                    "apikey" => AuthConfig::ApiKey,
                    "querykey" => AuthConfig::QueryKey,
                    _ => AuthConfig::Bearer,
                };

                ProviderConfig::new(&def.id, &def.name, &def.base_url)
                    .with_auth(auth)
                    .with_chat_path(&def.chat_path.unwrap_or_else(|| "/chat/completions".to_string()))
                    .with_embeddings_path(&def.embeddings_path.unwrap_or_else(|| "/embeddings".to_string()))
                    .with_openai_compatible(def.openai_compatible.unwrap_or(true))
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct CustomProviderDef {
    id: String,
    name: String,
    base_url: String,
    #[serde(default)]
    auth: String,
    #[serde(rename = "apiKeyEnv")]
    #[serde(default)]
    api_key_env: Option<String>,
    #[serde(rename = "chatPath")]
    #[serde(default)]
    chat_path: Option<String>,
    #[serde(rename = "embeddingsPath")]
    #[serde(default)]
    embeddings_path: Option<String>,
    #[serde(rename = "openaiCompatible")]
    #[serde(default)]
    openai_compatible: Option<bool>,
}

/// Provider registry that holds both built-in and custom providers
pub struct ProviderRegistry {
    providers: HashMap<String, ProviderConfig>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
        };
        registry.load_builtin();
        registry.load_custom();
        registry
    }

    fn load_builtin(&mut self) {
        for provider in BuiltinProviders::all() {
            self.providers.insert(provider.id.clone(), provider);
        }
    }

    fn load_custom(&mut self) {
        for provider in CustomProviders::load_from_env() {
            self.providers.insert(provider.id.clone(), provider);
        }
    }

    pub fn get(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.get(id)
    }

    pub fn list(&self) -> Vec<&ProviderConfig> {
        self.providers.values().collect()
    }

    pub fn register(&mut self, config: ProviderConfig) {
        self.providers.insert(config.id.clone(), config);
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}