//! 提供商模型单元测试
//!
//! 测试 Provider 结构的创建、验证和操作

use nexus_models::provider::{Provider, ProviderSlug};

#[test]
fn test_provider_creation() {
    let provider = Provider::new(
        "OpenAI".to_string(),
        ProviderSlug::OpenAI,
        "https://api.openai.com/v1".to_string(),
        1,
    );

    assert_eq!(provider.name, "OpenAI");
    assert_eq!(provider.slug, ProviderSlug::OpenAI);
    assert!(provider.is_active);
    assert_eq!(provider.priority, 1);
}

#[test]
fn test_provider_with_logo_url() {
    let provider = Provider::with_logo(
        "Anthropic".to_string(),
        ProviderSlug::Anthropic,
        "https://api.anthropic.com".to_string(),
        "https://example.com/logo.png".to_string(),
        2,
    );

    assert_eq!(provider.logo_url, Some("https://example.com/logo.png".to_string()));
}

#[test]
fn test_provider_slug_variants() {
    assert_eq!(ProviderSlug::OpenAI.as_str(), "openai");
    assert_eq!(ProviderSlug::Anthropic.as_str(), "anthropic");
    assert_eq!(ProviderSlug::Google.as_str(), "google");
    assert_eq!(ProviderSlug::DeepSeek.as_str(), "deepseek");
}

#[test]
fn test_provider_from_str() {
    assert_eq!(ProviderSlug::from_str("openai"), ProviderSlug::OpenAI);
    assert_eq!(ProviderSlug::from_str("anthropic"), ProviderSlug::Anthropic);
    assert_eq!(ProviderSlug::from_str("google"), ProviderSlug::Google);
    assert_eq!(ProviderSlug::from_str("deepseek"), ProviderSlug::DeepSeek);
    assert_eq!(ProviderSlug::from_str("unknown"), ProviderSlug::OpenAI); // 默认值
}

#[test]
fn test_provider_deactivate() {
    let mut provider = Provider::new(
        "Test Provider".to_string(),
        ProviderSlug::OpenAI,
        "https://api.test.com".to_string(),
        1,
    );

    assert!(provider.is_active);
    provider.is_active = false;
    assert!(!provider.is_active);
}

#[test]
fn test_provider_priority_ordering() {
    let mut p1 = Provider::new("Provider 1".to_string(), ProviderSlug::OpenAI, "https://api.test.com".to_string(), 3);
    let mut p2 = Provider::new("Provider 2".to_string(), ProviderSlug::Anthropic, "https://api.test.com".to_string(), 1);
    let mut p3 = Provider::new("Provider 3".to_string(), ProviderSlug::Google, "https://api.test.com".to_string(), 2);

    // 按优先级排序
    let mut providers = vec![&mut p1, &mut p2, &mut p3];
    providers.sort_by_key(|p| p.priority);

    assert_eq!(providers[0].priority, 1);
    assert_eq!(providers[1].priority, 2);
    assert_eq!(providers[2].priority, 3);
}
