//! Generic HTTP-based provider client
//!
//! Uses configuration-driven approach to call any LLM provider.
//! Supports both built-in and user-defined custom providers.

use crate::config::{AuthConfig, ProviderConfig, ProviderRegistry};
use crate::error::ProviderError;
use crate::providers::{get_registry, StreamChunk, StreamHandler};
use crate::types::{
    ChatRequest as ProviderChatRequest, ChatResponse, EmbeddingsRequest, EmbeddingsResponse,
    ProviderType,
};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use models::ProviderKey;

/// Trait for LLM provider clients
#[async_trait]
pub trait ProviderClient: Send + Sync {
    fn provider_type(&self) -> ProviderType;
    fn provider_id(&self) -> &str;
    fn key_id(&self) -> Option<uuid::Uuid>;
    async fn chat(&self, request: ProviderChatRequest) -> Result<ChatResponse, ProviderError>;
    async fn chat_stream(
        &self,
        request: ProviderChatRequest,
    ) -> Result<Vec<ChatChunk>, ProviderError>;
    async fn embeddings(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, ProviderError>;
}

/// Chat chunk for streaming
#[derive(Debug, Clone)]
pub struct ChatChunk {
    pub delta: String,
    pub finished: bool,
    pub finish_reason: Option<String>,
}

/// Generic HTTP provider client
pub struct HttpProviderClient {
    config: ProviderConfig,
    api_key: String,
    key_id: Option<uuid::Uuid>,
    client: Client,
}

impl HttpProviderClient {
    /// Create client using environment variable (legacy/standalone mode)
    pub fn new(provider_id: &str) -> Result<Self, ProviderError> {
        Self::new_with_key_id(provider_id, None)
    }

    /// Create client using a specific provider key (from database)
    pub fn new_with_key(provider_id: &str, key: &ProviderKey) -> Result<Self, ProviderError> {
        let registry = ProviderRegistry::new();
        let config = registry
            .get(provider_id)
            .ok_or_else(|| ProviderError::ProviderNotFound(provider_id.to_string()))?
            .clone();

        Ok(Self {
            config,
            api_key: key.api_key_encrypted.clone(),
            key_id: Some(key.id),
            client: Client::new(),
        })
    }

    /// Create client with key ID (key will be selected by caller)
    pub fn new_with_key_id(provider_id: &str, key_id: Option<uuid::Uuid>) -> Result<Self, ProviderError> {
        let registry = ProviderRegistry::new();
        let config = registry
            .get(provider_id)
            .ok_or_else(|| ProviderError::ProviderNotFound(provider_id.to_string()))?
            .clone();

        let api_key = Self::load_api_key_from_env(provider_id, &config)?;

        Ok(Self {
            config,
            api_key,
            key_id,
            client: Client::new(),
        })
    }

    fn load_api_key_from_env(provider_id: &str, config: &ProviderConfig) -> Result<String, ProviderError> {
        // For built-in providers, use well-known env vars
        // For custom providers, try CUSTOM_<ID>_API_KEY or just any available key
        let env_var = match provider_id {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            "google" => "GOOGLE_API_KEY",
            "deepseek" => "DEEPSEEK_API_KEY",
            _ => {
                // For custom providers, try CUSTOM_<ID>_API_KEY first
                let custom_env = format!("CUSTOM_{}_API_KEY", provider_id.to_uppercase().replace('-', "_"));
                if std::env::var(&custom_env).is_ok() {
                    return Ok(std::env::var(&custom_env).unwrap());
                }
                // Fallback to OPENAI_API_KEY for OpenAI-compatible custom providers
                if config.openai_compatible {
                    return std::env::var("OPENAI_API_KEY")
                        .map_err(|_| ProviderError::ApiKeyNotSet(provider_id.to_string()));
                }
                return Err(ProviderError::ApiKeyNotSet(provider_id.to_string()));
            }
        };

        std::env::var(env_var).map_err(|_| ProviderError::ApiKeyNotSet(provider_id.to_string()))
    }

    fn build_auth_header(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        match &self.config.auth {
            AuthConfig::Bearer => {
                headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));
            }
            AuthConfig::ApiKey => {
                headers.insert("x-api-key".to_string(), self.api_key.clone());
            }
            AuthConfig::QueryKey => {
                // Query key is handled in URL
            }
        }
        headers
    }

    fn build_url(&self, path: &str) -> String {
        match &self.config.auth {
            AuthConfig::QueryKey => format!("{}{}?key={}", self.config.base_url, path, self.api_key),
            _ => format!("{}{}", self.config.base_url, path),
        }
    }

    pub async fn chat(&self, request: ProviderChatRequest) -> Result<ChatResponse, ProviderError> {
        let url = self.build_url(&self.config.chat_path);

        let mut payload = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
        });

        if let Some(max_tokens) = request.max_tokens {
            payload["max_tokens"] = serde_json::json!(max_tokens);
        }

        // Apply provider-specific message transformation if needed
        let registry = get_registry().await;
        if let Some(transformer) = registry.get_transformer(&self.config.id) {
            let transformed = transformer.transform_messages(&request.messages);
            if let Some(msgs) = transformed.get("messages") {
                payload["messages"] = msgs.clone();
            }
            if let Some(system) = transformed.get("system") {
                payload["system"] = system.clone();
            }
        }

        let mut req = self.client.post(&url);

        // Add headers
        for (key, value) in &self.config.headers {
            req = req.header(key, value);
        }
        for (key, value) in self.build_auth_header() {
            req = req.header(&key, &value);
        }

        let start = Instant::now();
        let resp = req
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let latency_ms = start.elapsed().as_millis() as i32;
        let data: serde_json::Value = resp.json().await?;

        self.parse_chat_response(data, latency_ms)
    }

    pub async fn chat_stream(
        &self,
        request: ProviderChatRequest,
    ) -> Result<Vec<ChatChunk>, ProviderError> {
        let url = if let Some(stream_path) = &self.config.stream_path {
            self.build_url(stream_path)
        } else {
            self.build_url(&self.config.chat_path)
        };

        let mut payload = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "stream": true,
        });

        if let Some(max_tokens) = request.max_tokens {
            payload["max_tokens"] = serde_json::json!(max_tokens);
        }

        // Apply provider-specific message transformation if needed
        let registry = get_registry().await;
        if let Some(transformer) = registry.get_transformer(&self.config.id) {
            let transformed = transformer.transform_messages(&request.messages);
            if let Some(msgs) = transformed.get("messages") {
                payload["messages"] = msgs.clone();
            }
            if let Some(system) = transformed.get("system") {
                payload["system"] = system.clone();
            }
        }

        let mut req = self.client.post(&url);

        for (key, value) in &self.config.headers {
            req = req.header(key, value);
        }
        for (key, value) in self.build_auth_header() {
            req = req.header(&key, &value);
        }

        let mut chunks = Vec::new();

        let resp = req
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let stream_handler = registry
            .get_stream_handler(&self.config.id)
            .ok_or_else(|| ProviderError::StreamingNotSupported)?;

        let mut stream = resp.bytes_stream();

        while let Some(item) = stream.next().await {
            let bytes = item.map_err(|e| ProviderError::RequestFailed(e))?;
            let text = String::from_utf8(bytes.to_vec())
                .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

            for line in text.lines() {
                if let Some(chunk) = stream_handler.parse_sse_event(line) {
                    chunks.push(ChatChunk {
                        delta: chunk.delta,
                        finished: chunk.finished,
                        finish_reason: chunk.finish_reason,
                    });
                    if chunk.finished {
                        return Ok(chunks);
                    }
                }
            }
        }

        Ok(chunks)
    }

    pub async fn embeddings(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, ProviderError> {
        if self.config.embeddings_path.is_empty() {
            return Err(ProviderError::EmbeddingsNotSupported);
        }

        let url = self.build_url(&self.config.embeddings_path);

        let payload = if self.config.openai_compatible {
            serde_json::json!({
                "model": request.model,
                "input": request.inputs,
            })
        } else {
            // Provider-specific embedding request format
            serde_json::json!({
                "model": request.model,
                "content": {
                    "parts": request.inputs.iter().map(|s| serde_json::json!({"text": s})).collect::<Vec<_>>()
                }
            })
        };

        let mut req = self.client.post(&url);

        for (key, value) in &self.config.headers {
            req = req.header(key, value);
        }
        for (key, value) in self.build_auth_header() {
            req = req.header(&key, &value);
        }

        let resp = req
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        self.parse_embeddings_response(data)
    }

    fn parse_chat_response(&self, data: serde_json::Value, latency_ms: i32) -> Result<ChatResponse, ProviderError> {
        // Different providers have different response formats
        let (id, model, role, content) = if self.config.openai_compatible {
            let id = data["id"].as_str().unwrap_or("unknown").to_string();
            let model = data["model"].as_str().unwrap_or("").to_string();
            let role = data["choices"][0]["message"]["role"].as_str().unwrap_or("assistant");
            let content = data["choices"][0]["message"]["content"].as_str().unwrap_or("");
            (id, model, role.to_string(), content.to_string())
        } else if self.config.id == "anthropic" {
            let id = format!("msg_{}", data["id"].as_str().unwrap_or("unknown"));
            let model = data["model"].as_str().unwrap_or("").to_string();
            let content = data["content"][0]["text"].as_str().unwrap_or("");
            (id, model, "assistant".to_string(), content.to_string())
        } else if self.config.id == "google" {
            let id = format!("gemini_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());
            let model = data["model"].as_str().unwrap_or("").to_string();
            let content = data["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("");
            (id, model, "assistant".to_string(), content.to_string())
        } else {
            return Err(ProviderError::InvalidResponse(
                format!("Unsupported provider: {}", self.config.id)
            ));
        };

        let usage = data["usage"].clone();
        let prompt_tokens = usage["prompt_tokens"].as_i64().unwrap_or(0) as i32;
        let completion_tokens = usage["completion_tokens"].as_i64().unwrap_or(0) as i32;

        Ok(ChatResponse {
            id,
            model,
            message: crate::types::Message {
                role,
                content,
            },
            usage: [
                ("prompt_tokens".to_string(), prompt_tokens),
                ("completion_tokens".to_string(), completion_tokens),
                ("total_tokens".to_string(), prompt_tokens + completion_tokens),
            ]
            .into_iter()
            .collect(),
            latency_ms,
        })
    }

    fn parse_embeddings_response(&self, data: serde_json::Value) -> Result<EmbeddingsResponse, ProviderError> {
        let embeddings = if self.config.openai_compatible {
            data["data"]
                .as_array()
                .ok_or_else(|| ProviderError::InvalidResponse("Missing data array".to_string()))?
                .iter()
                .map(|item| {
                    item["embedding"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                        .collect()
                })
                .collect()
        } else if self.config.id == "google" {
            vec![data["embedding"]["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect()]
        } else {
            return Err(ProviderError::InvalidResponse("Unknown embedding format".to_string()));
        };

        Ok(EmbeddingsResponse { embeddings })
    }
}

#[async_trait]
impl ProviderClient for HttpProviderClient {
    fn provider_type(&self) -> ProviderType {
        ProviderType::from_str(&self.config.id).unwrap_or(ProviderType::OpenAI)
    }

    fn provider_id(&self) -> &str {
        &self.config.id
    }

    fn key_id(&self) -> Option<uuid::Uuid> {
        self.key_id
    }

    async fn chat(&self, request: ProviderChatRequest) -> Result<ChatResponse, ProviderError> {
        self.chat(request).await
    }

    async fn chat_stream(
        &self,
        request: ProviderChatRequest,
    ) -> Result<Vec<ChatChunk>, ProviderError> {
        self.chat_stream(request).await
    }

    async fn embeddings(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, ProviderError> {
        self.embeddings(request).await
    }
}

/// Provider client factory
pub struct ProviderClientFactory;

impl ProviderClientFactory {
    pub fn create(provider: &str) -> Result<Arc<dyn ProviderClient>, ProviderError> {
        Ok(Arc::new(HttpProviderClient::new(provider)?))
    }

    /// List all available providers (built-in + custom)
    pub fn list_providers() -> Vec<String> {
        let registry = ProviderRegistry::new();
        registry.list().iter().map(|p| p.id.clone()).collect()
    }

    /// Get provider configuration
    pub fn get_config(provider: &str) -> Option<ProviderConfig> {
        let registry = ProviderRegistry::new();
        registry.get(provider).cloned()
    }
}