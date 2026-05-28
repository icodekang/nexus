//! 用户自托管 Provider API Key (BYOK)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProviderKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider_slug: String,
    pub name: Option<String>,
    pub api_key_encrypted: String,
    pub api_key_prefix: String,
    pub base_url: String,
    pub is_active: bool,
    pub priority_level: PriorityLevel,
    pub sort_order: i32,
    pub always_use: bool,
    pub model_filter: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PriorityLevel {
    Prioritized,
    Fallback,
}

impl PriorityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PriorityLevel::Prioritized => "prioritized",
            PriorityLevel::Fallback => "fallback",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fallback" => PriorityLevel::Fallback,
            _ => PriorityLevel::Prioritized,
        }
    }
}

impl UserProviderKey {
    pub fn new(
        user_id: Uuid,
        provider_slug: String,
        api_key_encrypted: String,
        api_key_prefix: String,
        base_url: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            provider_slug,
            name: None,
            api_key_encrypted,
            api_key_prefix,
            base_url,
            is_active: true,
            priority_level: PriorityLevel::Prioritized,
            sort_order: 0,
            always_use: false,
            model_filter: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_priority(mut self, level: PriorityLevel, sort_order: i32) -> Self {
        self.priority_level = level;
        self.sort_order = sort_order;
        self
    }

    pub fn mask_key(&self) -> String {
        let prefix = &self.api_key_prefix;
        if prefix.len() <= 4 {
            format!("{}...", prefix)
        } else {
            format!("{}...{}", &prefix[..4], &prefix[prefix.len()-4..])
        }
    }
}
