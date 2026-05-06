pub mod defs;
pub mod parser;
pub mod prompt;

use crate::client::{ChatChunk, ProviderClient};
use crate::error::ProviderError;
use crate::types::{ChatRequest, ChatResponse, Message};
use async_trait::async_trait;
use defs::ToolDef;
use parser::{extract_tool_calls, ToolCall};
use std::sync::Arc;

pub use defs::default_tools;
pub use prompt::inject_tool_prompt;

const MAX_TOOL_ROUNDS: usize = 8;

pub struct ToolCallMiddleware {
    inner: Arc<dyn ProviderClient>,
    tools: Vec<ToolDef>,
}

impl ToolCallMiddleware {
    pub fn new(inner: Arc<dyn ProviderClient>, tools: Vec<ToolDef>) -> Self {
        Self { inner, tools }
    }

    pub async fn chat_with_tools(
        &self,
        request: ChatRequest,
        provider: &str,
    ) -> Result<ChatResponse, ProviderError> {
        let mut all_content = String::new();
        let mut messages_for_model = request.messages.clone();

        for round in 0..MAX_TOOL_ROUNDS {
            let injected = inject_tool_prompt(&messages_for_model, &self.tools, provider);

            let mut round_request = request.clone();
            round_request.messages = injected;

            let response = self.inner.chat(round_request).await?;

            let response_text = &response.message.content;

            if !parser::has_tool_call(response_text) {
                all_content.push_str(response_text);
                return Ok(ChatResponse {
                    id: response.id,
                    model: response.model,
                    message: Message::assistant(all_content),
                    usage: response.usage,
                    latency_ms: response.latency_ms,
                });
            }

            let calls = extract_tool_calls(response_text);
            if calls.is_empty() {
                all_content.push_str(response_text);
                return Ok(ChatResponse {
                    id: response.id,
                    model: response.model,
                    message: Message::assistant(all_content),
                    usage: response.usage,
                    latency_ms: response.latency_ms,
                });
            }

            tracing::info!(
                "Round {}: model requested {} tool call(s)",
                round + 1,
                calls.len()
            );

            for call in &calls {
                let tool_result = execute_tool(call);
                all_content.push_str(&format!(
                    "\n[Tool: {}] {}\n",
                    call.tool, tool_result
                ));

                messages_for_model.push(Message::assistant(format!(
                    "```tool_json\n{{\"tool\": \"{}\", \"args\": {}}}\n```",
                    call.tool,
                    serde_json::to_string(&call.args).unwrap_or_default()
                )));
                messages_for_model.push(Message::user(format!(
                    "Tool {} result:\n{}",
                    call.tool, tool_result
                )));
            }
        }

        Ok(ChatResponse {
            id: format!("tool-{}", uuid::Uuid::new_v4()),
            model: request.model.clone(),
            message: Message::assistant(all_content),
            usage: Default::default(),
            latency_ms: 0,
        })
    }

    pub async fn chat_stream_with_tools(
        &self,
        request: ChatRequest,
        provider: &str,
    ) -> Result<Vec<ChatChunk>, ProviderError> {
        let response = self.chat_with_tools(request, provider).await?;
        Ok(vec![ChatChunk {
            delta: response.message.content.clone(),
            finished: true,
            finish_reason: Some("stop".to_string()),
        }])
    }
}

fn execute_tool(call: &ToolCall) -> String {
    match call.tool.as_str() {
        "web_search" => {
            let query = call.args.get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!("[web_search not available offline] query: {}", query)
        }
        "read" => {
            let path = call.args.get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            std::fs::read_to_string(path)
                .map(|s| s.chars().take(4000).collect::<String>())
                .unwrap_or_else(|e| format!("Error reading: {}", e))
        }
        "write" => {
            let path = call.args.get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let content = call.args.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match std::fs::write(path, content) {
                Ok(()) => format!("Written to {}", path),
                Err(e) => format!("Error writing: {}", e),
            }
        }
        "exec" => {
            let command = call.args.get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    let mut result = stdout.to_string();
                    if !stderr.is_empty() {
                        result.push_str("\nstderr: ");
                        result.push_str(&stderr);
                    }
                    result.chars().take(4000).collect()
                }
                Err(e) => format!("Error executing: {}", e),
            }
        }
        "message" => {
            call.args.get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        _ => format!("Unknown tool: {}", call.tool),
    }
}

#[async_trait]
impl ProviderClient for ToolCallMiddleware {
    fn provider_type(&self) -> crate::types::ProviderType {
        self.inner.provider_type()
    }

    fn provider_id(&self) -> &str {
        self.inner.provider_id()
    }

    fn key_id(&self) -> Option<uuid::Uuid> {
        self.inner.key_id()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        self.chat_with_tools(request, self.provider_id()).await
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Vec<ChatChunk>, ProviderError> {
        self.chat_stream_with_tools(request, self.provider_id()).await
    }

    async fn embeddings(
        &self,
        request: crate::types::EmbeddingsRequest,
    ) -> Result<crate::types::EmbeddingsResponse, ProviderError> {
        self.inner.embeddings(request).await
    }
}

pub fn wrap_with_tool_calling(
    client: Arc<dyn ProviderClient>,
    tools: Vec<ToolDef>,
) -> Arc<dyn ProviderClient> {
    Arc::new(ToolCallMiddleware::new(client, tools))
}
