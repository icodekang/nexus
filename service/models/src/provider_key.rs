use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Encrypted API key for an LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderKey {
    pub id: Uuid,
    pub provider_slug: String,
    pub api_key_encrypted: String,
    pub api_key_prefix: String,
    pub base_url: String,
    pub is_active: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProviderKey {
    pub fn new(provider_slug: String, api_key_encrypted: String, api_key_prefix: String, base_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider_slug,
            api_key_encrypted,
            api_key_prefix,
            base_url,
            is_active: true,
            priority: 100,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Generate a masked display of the API key, e.g. "sk-...a1b2"
    pub fn masked_key(&self) -> String {
        if self.api_key_prefix.len() >= 4 {
            format!("{}...{}", &self.api_key_prefix[..4], &self.api_key_prefix[self.api_key_prefix.len()-4..])
        } else {
            format!("{}...", self.api_key_prefix)
        }
    }
}
