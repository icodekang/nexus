use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// LLM Provider (OpenAI, Anthropic, Google, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub api_base_url: String,
    pub is_active: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
}

impl Provider {
    pub fn new(name: String, slug: String, api_base_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            slug,
            logo_url: None,
            api_base_url,
            is_active: true,
            priority: 100,
            created_at: Utc::now(),
        }
    }

    pub fn with_logo(mut self, logo_url: String) -> Self {
        self.logo_url = Some(logo_url);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Built-in providers
pub struct Providers;

impl Providers {
    pub const OPENAI: &'static str = "openai";
    pub const ANTHROPIC: &'static str = "anthropic";
    pub const GOOGLE: &'static str = "google";
    pub const DEEPSEEK: &'static str = "deepseek";
    
    pub fn all() -> Vec<Provider> {
        vec![
            Provider::new(
                "OpenAI".to_string(),
                Self::OPENAI.to_string(),
                "https://api.openai.com/v1".to_string(),
            )
            .with_priority(10),
            Provider::new(
                "Anthropic".to_string(),
                Self::ANTHROPIC.to_string(),
                "https://api.anthropic.com/v1".to_string(),
            )
            .with_priority(20),
            Provider::new(
                "Google".to_string(),
                Self::GOOGLE.to_string(),
                "https://generativelanguage.googleapis.com/v1beta".to_string(),
            )
            .with_priority(30),
            Provider::new(
                "DeepSeek".to_string(),
                Self::DEEPSEEK.to_string(),
                "https://api.deepseek.com/v1".to_string(),
            )
            .with_priority(40),
        ]
    }
}
