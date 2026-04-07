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

    /// Extract prefix from a plain key
    pub fn extract_prefix(key: &str) -> Option<String> {
        key.split('-').next().map(|s| s.to_string())
    }

    /// Validate key format
    pub fn validate_format(key: &str) -> bool {
        let parts: Vec<&str> = key.split('-').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let prefix = parts[0];
        let random_part = parts[1];
        
        prefix.starts_with("sk-") && random_part.len() == 32
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
    }
}
