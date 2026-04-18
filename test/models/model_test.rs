//! AI 模型单元测试
//!
//! 测试 LlmModel 结构的创建、配置和能力

use nexus_models::model::{LlmModel, ModelMode, ModelCapability};

#[test]
fn test_model_creation() {
    let model = LlmModel::new(
        "gpt-4".to_string(),
        "GPT-4".to_string(),
        ProviderSlug::OpenAI,
        "gpt-4".to_string(),
        ModelMode::Chat,
        128000,
    );

    assert_eq!(model.name, "GPT-4");
    assert_eq!(model.model_id, "gpt-4");
    assert_eq!(model.context_window, 128000);
    assert!(model.is_active);
}

#[test]
fn test_model_default_capabilities() {
    let model = LlmModel::new(
        "gpt-4".to_string(),
        "GPT-4".to_string(),
        ProviderSlug::OpenAI,
        "gpt-4".to_string(),
        ModelMode::Chat,
        128000,
    );

    // 默认应包含 chat 能力
    assert!(model.capabilities.contains(&ModelCapability::Chat));
}

#[test]
fn test_model_with_custom_capabilities() {
    let model = LlmModel::with_capabilities(
        "claude-3".to_string(),
        "Claude 3".to_string(),
        ProviderSlug::Anthropic,
        "claude-3-opus".to_string(),
        ModelMode::Chat,
        200000,
        vec![ModelCapability::Vision, ModelCapability::Function],
    );

    assert!(model.capabilities.contains(&ModelCapability::Vision));
    assert!(model.capabilities.contains(&ModelCapability::Function));
    assert!(!model.capabilities.contains(&ModelCapability::Embedding));
}

#[test]
fn test_model_mode() {
    assert_eq!(ModelMode::Chat.as_str(), "chat");
    assert_eq!(ModelMode::Completion.as_str(), "completion");
    assert_eq!(ModelMode::Embedding.as_str(), "embedding");
}

#[test]
fn test_model_capability() {
    assert_eq!(ModelCapability::Chat.as_str(), "chat");
    assert_eq!(ModelCapability::Vision.as_str(), "vision");
    assert_eq!(ModelCapability::Function.as_str(), "function");
    assert_eq!(ModelCapability::Embedding.as_str(), "embedding");
}

#[test]
fn test_model_context_window_scaling() {
    let model_8k = LlmModel::new(
        "gpt-4".to_string(),
        "GPT-4-8K".to_string(),
        ProviderSlug::OpenAI,
        "gpt-4".to_string(),
        ModelMode::Chat,
        8192,
    );

    let model_128k = LlmModel::new(
        "gpt-4-32k".to_string(),
        "GPT-4-32K".to_string(),
        ProviderSlug::OpenAI,
        "gpt-4-32k".to_string(),
        ModelMode::Chat,
        32768,
    );

    assert!(model_128k.context_window > model_8k.context_window);
}

#[test]
fn test_model_deactivate() {
    let mut model = LlmModel::new(
        "test-model".to_string(),
        "Test Model".to_string(),
        ProviderSlug::OpenAI,
        "test-model".to_string(),
        ModelMode::Chat,
        4096,
    );

    assert!(model.is_active);
    model.is_active = false;
    assert!(!model.is_active);
}

#[test]
fn test_model_serialization() {
    let model = LlmModel::with_capabilities(
        "gpt-4".to_string(),
        "GPT-4".to_string(),
        ProviderSlug::OpenAI,
        "gpt-4".to_string(),
        ModelMode::Chat,
        128000,
        vec![ModelCapability::Chat, ModelCapability::Function],
    );

    let json = serde_json::to_string(&model).unwrap();
    assert!(json.contains("GPT-4"));
    assert!(json.contains("openai"));

    let deserialized: LlmModel = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, model.name);
}
