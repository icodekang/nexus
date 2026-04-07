use uuid::Uuid;
use sha2::{Sha256, Digest};

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
    pub fn generate(&self) -> (String, String) {
        // Generate random key
        let random_part = Uuid::new_v4().to_string().replace("-", "");
        let key = format!("{}-{}", self.prefix, random_part);

        // Hash the key for storage
        let hash = Self::hash_key(&key);

        (key, hash)
    }

    /// Hash a key for secure storage
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify a key against its hash
    pub fn verify(key: &str, hash: &str) -> bool {
        Self::hash_key(key) == hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_hash() {
        let gen = ApiKeyGenerator::new("sk-nova");
        let (key, hash) = gen.generate();
        assert!(key.starts_with("sk-nova-"));
        assert!(ApiKeyGenerator::verify(&key, &hash));
    }
}
