//! 聊天模块
//!
//! 定义了聊天完成和嵌入请求/响应的数据结构

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 角色（user/assistant/system）
    pub role: String,
    /// 消息内容
    pub content: String,
    /// 名称（可选，用于 function 调用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            name: None,
        }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            name: None,
        }
    }

    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            name: None,
        }
    }
}

/// 聊天完成选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// 选项索引
    pub index: usize,
    /// 消息
    pub message: Message,
    /// 完成原因（stop/length）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Token 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// 输入 Token 数量
    pub prompt_tokens: i32,
    /// 输出 Token 数量
    pub completion_tokens: i32,
    /// 总 Token 数量
    pub total_tokens: i32,
}

impl Usage {
    /// 创建新的使用统计
    pub fn new(prompt_tokens: i32, completion_tokens: i32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// 聊天完成请求（OpenAI 兼容格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// 模型标识符
    pub model: String,
    /// 消息列表
    pub messages: Vec<Message>,
    /// 生成温度
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// 最大输出 Token 数
    #[serde(default)]
    pub max_tokens: Option<i32>,
    /// 是否流式响应
    #[serde(default)]
    pub stream: bool,
    /// Top-P 采样
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// 停止序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// 存在惩罚
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// 频率惩罚
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// 用户标识（用于跟踪）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

fn default_temperature() -> f32 {
    0.7
}

impl ChatRequest {
    /// 创建新的聊天请求
    ///
    /// # 参数
    /// * `model` - 模型标识符
    /// * `messages` - 消息列表
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: 0.7,
            max_tokens: None,
            stream: false,
            top_p: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        }
    }
}

/// 聊天完成响应（OpenAI 兼容格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// 响应 ID
    pub id: String,
    /// 对象类型
    pub object: String,
    /// 创建时间戳
    pub created: u64,
    /// 模型标识符
    pub model: String,
    /// 完成选项列表
    pub choices: Vec<Choice>,
    /// Token 使用统计
    pub usage: Usage,
}

impl ChatResponse {
    /// 创建新的聊天响应
    ///
    /// # 参数
    /// * `model` - 模型标识符
    /// * `message` - 响应消息
    /// * `usage` - Token 使用统计
    pub fn new(model: impl Into<String>, message: Message, usage: Usage) -> Self {
        Self {
            id: format!("chatcmpl-{}", Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp() as u64,
            model: model.into(),
            choices: vec![Choice {
                index: 0,
                message,
                finish_reason: Some("stop".to_string()),
            }],
            usage,
        }
    }
}

/// 流式响应块（SSE 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    /// 块 ID
    pub id: String,
    /// 对象类型
    pub object: String,
    /// 创建时间戳
    pub created: u64,
    /// 模型标识符
    pub model: String,
    /// 选项列表
    pub choices: Vec<ChunkChoice>,
}

/// 流式选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkChoice {
    /// 选项索引
    pub index: usize,
    /// 增量内容
    pub delta: ChunkDelta,
    /// 完成原因
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// 流式增量内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkDelta {
    /// 角色
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// 内容增量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

impl ChatChunk {
    /// 创建新的聊天块
    ///
    /// # 参数
    /// * `model` - 模型标识符
    /// * `content` - 内容增量
    /// * `finished` - 是否完成
    pub fn new(model: impl Into<String>, content: impl Into<String>, finished: bool) -> Self {
        Self {
            id: format!("chatcmpl-{}", Uuid::new_v4()),
            object: "chat.completion.chunk".to_string(),
            created: Utc::now().timestamp() as u64,
            model: model.into(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: ChunkDelta {
                    role: None,
                    content: Some(content.into()),
                },
                finish_reason: if finished { Some("stop".to_string()) } else { None },
            }],
        }
    }
}

/// 嵌入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsRequest {
    /// 模型标识符
    pub model: String,
    /// 输入文本列表
    pub input: Vec<String>,
    /// 用户标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl EmbeddingsRequest {
    /// 创建新的嵌入请求
    ///
    /// # 参数
    /// * `model` - 模型标识符
    /// * `input` - 输入文本列表
    pub fn new(model: impl Into<String>, input: Vec<String>) -> Self {
        Self {
            model: model.into(),
            input,
            user: None,
        }
    }
}

/// 嵌入响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsResponse {
    /// 对象类型
    pub object: String,
    /// 嵌入数据列表
    pub data: Vec<EmbeddingData>,
    /// 模型标识符
    pub model: String,
    /// Token 使用统计
    pub usage: Usage,
}

/// 单个嵌入数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    /// 对象类型
    pub object: String,
    /// 嵌入向量
    pub embedding: Vec<f32>,
    /// 索引
    pub index: usize,
}

impl EmbeddingsResponse {
    /// 创建新的嵌入响应
    ///
    /// # 参数
    /// * `model` - 模型标识符
    /// * `embeddings` - 嵌入向量列表
    pub fn new(model: impl Into<String>, embeddings: Vec<Vec<f32>>) -> Self {
        let usage = Usage::new(
            embeddings.iter().map(|e| e.len() as i32).sum(),
            0,
        );
        Self {
            object: "list".to_string(),
            data: embeddings
                .into_iter()
                .enumerate()
                .map(|(index, embedding)| EmbeddingData {
                    object: "embedding".to_string(),
                    embedding,
                    index,
                })
                .collect(),
            model: model.into(),
            usage,
        }
    }
}

// ── 批量查询（多模型对比） ──────────────────────────────────────────────────

/// 批量聊天请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchChatRequest {
    /// 消息列表
    pub messages: Vec<Message>,
    /// 指定模型列表（可选，不传则系统智能选择）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    /// 最大输出 Token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
}

/// 单个模型的查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResult {
    /// 模型标识
    pub model: String,
    /// Provider 标识
    pub provider: String,
    /// 回答内容
    pub content: String,
    /// 评分（1.0-1.0）
    pub score: f64,
    /// 评分理由
    pub reason: String,
    /// 请求延迟（毫秒）
    pub latency_ms: u64,
    /// Token 使用统计
    pub usage: Usage,
    /// 是否成功
    pub success: bool,
    /// 错误信息（失败时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 批量聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchChatResponse {
    /// 响应 ID
    pub id: String,
    /// 用户问题
    pub query: String,
    /// 各模型结果（未排序，前端先随机展示）
    pub results: Vec<ModelResult>,
    /// 评分使用的模型（仅在 has_scoring 为 true 时有值）
    pub judge_model: String,
    /// 总耗时（毫秒）
    pub total_latency_ms: u64,
    /// 问题分类（code / creative / analysis / general）
    pub selection_category: String,
    /// 被选中的模型 slug 列表
    pub selected_models: Vec<String>,
    /// 是否有评分环节（available_models >= 4 时启用）
    pub has_scoring: bool,
}

/// 批量评分请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJudgeRequest {
    /// 用户问题
    pub query: String,
    /// 各模型结果
    pub results: Vec<ModelResult>,
}

/// 批量评分响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJudgeResponse {
    /// 评分结果列表
    pub scores: Vec<JudgeScoreInfo>,
    /// 评委模型
    pub judge_model: String,
}

/// 单个评分信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeScoreInfo {
    /// 模型标识
    pub model: String,
    /// 评分
    pub score: f64,
    /// 评分理由
    pub reason: String,
}
