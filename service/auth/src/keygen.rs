use sha2::{Sha256, Digest};
use uuid::Uuid;

/// API Key generator
pub struct ApiKeyGenerator {
    prefix: String,
}

impl ApiKeyGenerator {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    /// Generate a new API key
    /// Returns (plain_key, hashed_key)
    /// Key format: sk-nexus-{32_hex_chars}
    pub fn generate(&self) -> (String, String) {
        let random_part = Uuid::new_v4().to_string().replace("-", "");
        let plain_key = format!("{}-{}", self.prefix, random_part);
        let hashed_key = Self::hash_key(&plain_key);

        (plain_key, hashed_key)
    }

    /// Hash a key for storage (SHA256)
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify a key against its hash
    pub fn verify(key: &str, hash: &str) -> bool {
        Self::hash_key(key) == hash
    }

    /// Extract prefix from a plain key (e.g. "sk-nexus" from "sk-nexus-abc123")
    pub fn extract_prefix(key: &str) -> Option<String> {
        // Format is: {service_prefix}-{group_prefix}-{random}
        // e.g. "sk-nexus-abc123" -> prefix is "sk-nexus"
        let parts: Vec<&str> = key.split('-').collect();
        if parts.len() < 3 {
            return None;
        }
        // Join first two parts with '-'
        Some(format!("{}-{}", parts[0], parts[1]))
    }

    /// Validate key format
    /// Expected: sk-nexus-{32_hex_chars}
    pub fn validate_format(key: &str) -> bool {
        // Must start with "sk-"
        if !key.starts_with("sk-") {
            return false;
        }

        // Split into at least 3 parts: ["sk", "nexus", "{32hex}", ...]
        let parts: Vec<&str> = key.splitn(4, '-').collect();
        if parts.len() < 3 {
            return false;
        }

        // The random part (3rd segment) must be 32 hex chars
        let random_part = parts[2];
        random_part.len() == 32 && random_part.chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let gen = ApiKeyGenerator::new("sk-nexus");
        let (plain, hashed) = gen.generate();

        assert!(plain.starts_with("sk-nexus-"));
        assert_eq!(hashed.len(), 64); // SHA256 produces 64 hex chars
        assert!(ApiKeyGenerator::verify(&plain, &hashed));
    }

    #[test]
    fn test_validate_format() {
        let gen = ApiKeyGenerator::new("sk-nexus");
        let (key, _) = gen.generate();

        assert!(ApiKeyGenerator::validate_format(&key));
        assert!(!ApiKeyGenerator::validate_format("invalid"));
        assert!(!ApiKeyGenerator::validate_format("sk-short"));
        assert!(!ApiKeyGenerator::validate_format("sk-nexus-short"));
        assert!(!ApiKeyGenerator::validate_format("wrong-prefix-abcdef1234567890abcdef1234567890"));
    }

    #[test]
    fn test_extract_prefix() {
        assert_eq!(
            ApiKeyGenerator::extract_prefix("sk-nexus-abcdef1234567890abcdef1234567890"),
            Some("sk-nexus".to_string())
        );
        assert_eq!(ApiKeyGenerator::extract_prefix("sk-only"), None);
        assert_eq!(ApiKeyGenerator::extract_prefix("invalid"), None);
    }

    #[test]
    fn test_verify_roundtrip() {
        let gen = ApiKeyGenerator::new("sk-nexus");
        let (plain, hashed) = gen.generate();
        assert!(ApiKeyGenerator::verify(&plain, &hashed));
        assert!(!ApiKeyGenerator::verify("wrong-key", &hashed));
    }
}
