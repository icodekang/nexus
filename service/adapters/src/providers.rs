//! Provider registry - allows dynamic registration of provider adapters
//!
//! Providers can register custom message transformers and stream handlers.
//! This is useful for providers with non-standard API formats.

use crate::error::ProviderError;
use crate::types::Message;
use std::collections::HashMap;
use std::sync::RwLock;

/// Trait for provider-specific message transformation
/// Only providers with non-standard message formats need to implement this
pub trait MessageTransformer: Send + Sync {
    /// Transform messages from standard format to provider-specific format
    fn transform_messages(&self, messages: &[Message]) -> serde_json::Value;
}

/// Streaming response chunk
pub struct StreamChunk {
    pub delta: String,
    pub finished: bool,
    pub finish_reason: Option<String>,
}

/// Trait for streaming response handling
pub trait StreamHandler: Send + Sync {
    /// Parse a complete SSE event (may span multiple lines).
    /// Returns None if the event should be skipped (comments, empty, etc.)
    fn parse_sse_event(&self, event: &str) -> Option<StreamChunk>;
}

/// Standard OpenAI-compatible stream handler
pub struct OpenAIStreamHandler;

impl StreamHandler for OpenAIStreamHandler {
    fn parse_sse_event(&self, event: &str) -> Option<StreamChunk> {
        // Extract data line from SSE event
        let data = extract_sse_data(event)?;
        if data == "[DONE]" {
            return Some(StreamChunk {
                delta: String::new(),
                finished: true,
                finish_reason: Some("stop".to_string()),
            });
        }
        let parsed: serde_json::Value = serde_json::from_str(data).ok()?;
        let delta = parsed["choices"][0]["delta"]["content"].as_str().unwrap_or("");
        let finish_reason = parsed["choices"][0]["finish_reason"].as_str().map(|s| s.to_string());
        Some(StreamChunk {
            delta: delta.to_string(),
            finished: finish_reason.is_some(),
            finish_reason,
        })
    }
}

/// Anthropic Messages API stream handler
/// SSE format:
///   event: content_block_delta
///   data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"hello"}}
///
///   event: message_stop
///   data: {"type":"message_stop"}
pub struct AnthropicStreamHandler;

impl StreamHandler for AnthropicStreamHandler {
    fn parse_sse_event(&self, event: &str) -> Option<StreamChunk> {
        let data = extract_sse_data(event)?;

        let parsed: serde_json::Value = serde_json::from_str(data).ok()?;
        let msg_type = parsed["type"].as_str().unwrap_or("");

        match msg_type {
            "content_block_delta" => {
                let text = parsed["delta"]["text"].as_str().unwrap_or("");
                Some(StreamChunk {
                    delta: text.to_string(),
                    finished: false,
                    finish_reason: None,
                })
            }
            "message_delta" => {
                let stop_reason = parsed["delta"]["stop_reason"].as_str().map(|s| s.to_string());
                Some(StreamChunk {
                    delta: String::new(),
                    finished: false,
                    finish_reason: stop_reason,
                })
            }
            "message_stop" => {
                Some(StreamChunk {
                    delta: String::new(),
                    finished: true,
                    finish_reason: Some("stop".to_string()),
                })
            }
            _ => None, // Skip message_start, content_block_start, ping, etc.
        }
    }
}

/// Google Gemini API stream handler
/// SSE format:
///   data: {"candidates":[{"content":{"parts":[{"text":"hello"}],"role":"model"}}]}
pub struct GoogleStreamHandler;

impl StreamHandler for GoogleStreamHandler {
    fn parse_sse_event(&self, event: &str) -> Option<StreamChunk> {
        let data = extract_sse_data(event)?;
        let parsed: serde_json::Value = serde_json::from_str(data).ok()?;

        // Check for finish reason
        let finish_reason = parsed["candidates"][0]["finishReason"]
            .as_str()
            .map(|s| s.to_string());

        let text = parsed["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("");

        Some(StreamChunk {
            delta: text.to_string(),
            finished: finish_reason.is_some() && finish_reason.as_deref() != Some(""),
            finish_reason,
        })
    }
}

/// Extract the `data:` payload from an SSE event block.
/// SSE events can have multiple lines (event:, data:, id:, etc.)
/// We only care about the `data:` line(s).
fn extract_sse_data(event: &str) -> Option<&str> {
    for line in event.lines() {
        let line = line.trim();
        if let Some(data) = line.strip_prefix("data: ") {
            return Some(data);
        }
    }
    None
}

/// Provider adapter registry
pub struct ProviderAdapterRegistry {
    transformers: HashMap<String, Box<dyn MessageTransformer>>,
    stream_handlers: HashMap<String, Box<dyn StreamHandler>>,
}

impl Default for ProviderAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderAdapterRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            transformers: HashMap::new(),
            stream_handlers: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    fn register_defaults(&mut self) {
        // Register built-in transformers
        self.register_transformer("anthropic", Box::new(AnthropicTransformer));
        self.register_transformer("google", Box::new(GoogleTransformer));

        // Register stream handlers
        self.register_stream_handler("openai", Box::new(OpenAIStreamHandler));
        self.register_stream_handler("deepseek", Box::new(OpenAIStreamHandler));
        self.register_stream_handler("anthropic", Box::new(AnthropicStreamHandler));
        self.register_stream_handler("google", Box::new(GoogleStreamHandler));
    }

    pub fn register_transformer(&mut self, provider: &str, transformer: Box<dyn MessageTransformer>) {
        self.transformers.insert(provider.to_string(), transformer);
    }

    pub fn register_stream_handler(&mut self, provider: &str, handler: Box<dyn StreamHandler>) {
        self.stream_handlers.insert(provider.to_string(), handler);
    }

    pub fn get_transformer(&self, provider: &str) -> Option<&dyn MessageTransformer> {
        self.transformers.get(provider).map(|b| b.as_ref())
    }

    pub fn get_stream_handler(&self, provider: &str) -> Option<&dyn StreamHandler> {
        self.stream_handlers.get(provider).map(|b| b.as_ref())
    }

    /// Check if a provider needs a custom transformer
    pub fn needs_transformer(&self, provider: &str) -> bool {
        matches!(provider, "anthropic" | "google")
    }

    /// Check if a provider needs a custom stream handler
    pub fn needs_stream_handler(&self, provider: &str) -> bool {
        !matches!(provider, "openai" | "deepseek")
    }
}

// Built-in transformers

struct AnthropicTransformer;

impl MessageTransformer for AnthropicTransformer {
    fn transform_messages(&self, messages: &[Message]) -> serde_json::Value {
        let mut anthropic_messages = Vec::new();
        let mut system_prompt = String::new();

        for msg in messages {
            match msg.role.as_str() {
                "system" => system_prompt = msg.content.clone(),
                "user" => {
                    anthropic_messages.push(serde_json::json!({
                        "role": "user",
                        "content": msg.content,
                    }));
                }
                "assistant" => {
                    anthropic_messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": msg.content,
                    }));
                }
                _ => {}
            }
        }

        let mut result = serde_json::json!({
            "messages": anthropic_messages,
        });
        if !system_prompt.is_empty() {
            result["system"] = serde_json::Value::String(system_prompt);
        }
        result
    }
}

struct GoogleTransformer;

impl MessageTransformer for GoogleTransformer {
    fn transform_messages(&self, messages: &[Message]) -> serde_json::Value {
        let contents: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|msg| {
                let role = match msg.role.as_str() {
                    "user" => "user",
                    "assistant" => "model",
                    _ => "user",
                };
                serde_json::json!({
                    "role": role,
                    "parts": [{"text": msg.content}]
                })
            })
            .collect();

        serde_json::json!({ "contents": contents })
    }
}

/// Global registry instance
static REGISTRY: RwLock<ProviderAdapterRegistry> = RwLock::new(ProviderAdapterRegistry::new());

pub fn get_registry() -> &'static ProviderAdapterRegistry {
    &REGISTRY
}

/// Register a new provider transformer at runtime
pub fn register_transformer(provider: &str, transformer: Box<dyn MessageTransformer>) -> Result<(), ProviderError> {
    let mut registry = REGISTRY.write().map_err(|_| ProviderError::InvalidResponse("Registry lock error".to_string()))?;
    registry.register_transformer(provider, transformer);
    Ok(())
}

/// Register a new provider stream handler at runtime
pub fn register_stream_handler(provider: &str, handler: Box<dyn StreamHandler>) -> Result<(), ProviderError> {
    let mut registry = REGISTRY.write().map_err(|_| ProviderError::InvalidResponse("Registry lock error".to_string()))?;
    registry.register_stream_handler(provider, handler);
    Ok(())
}
