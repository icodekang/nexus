//! API Key 生成器模块
//!
//! 提供 API Key 的生成、哈希和验证功能

use sha2::{Sha256, Digest};
use uuid::Uuid;

/// API Key 生成器
///
/// 用于生成格式为 `sk-nexus-{32_hex_chars}` 的 API Key
pub struct ApiKeyGenerator {
    /// Key 前缀
    prefix: String,
}

impl ApiKeyGenerator {
    /// 创建新的生成器
    ///
    /// # 参数
    /// * `prefix` - Key 前缀（如 "sk-nexus"）
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    /// 生成新的 API Key
    ///
    /// # 返回
    /// - `(plain_key, hashed_key)`: 明文 Key 和哈希后的 Key
    ///
    /// # Key 格式
    /// `sk-nexus-{32_hex_chars}`
    pub fn generate(&self) -> (String, String) {
        let random_part = Uuid::new_v4().to_string().replace("-", "");
        let plain_key = format!("{}-{}", self.prefix, random_part);
        let hashed_key = Self::hash_key(&plain_key);

        (plain_key, hashed_key)
    }

    /// 对 Key 进行哈希（用于存储）
    ///
    /// 使用 SHA256 算法
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 验证 Key 与哈希是否匹配
    ///
    /// # 参数
    /// * `key` - 明文 Key
    /// * `hash` - 存储的哈希值
    pub fn verify(key: &str, hash: &str) -> bool {
        Self::hash_key(key) == hash
    }

    /// 从明文 Key 中提取前缀
    ///
    /// # 示例
    /// `"sk-nexus-abc123"` → `"sk-nexus"`
    pub fn extract_prefix(key: &str) -> Option<String> {
        // 格式: {service_prefix}-{group_prefix}-{random}
        // 例如 "sk-nexus-abc123" -> 前缀是 "sk-nexus"
        let parts: Vec<&str> = key.split('-').collect();
        if parts.len() < 3 {
            return None;
        }
        // 将前两部分用 '-' 连接
        Some(format!("{}-{}", parts[0], parts[1]))
    }

    /// 验证 Key 格式是否正确
    ///
    /// # 期望格式
    /// `sk-nexus-{32_hex_chars}`
    pub fn validate_format(key: &str) -> bool {
        // 必须以 "sk-" 开头
        if !key.starts_with("sk-") {
            return false;
        }

        // 分割成至少 3 部分: ["sk", "nexus", "{32hex}", ...]
        let parts: Vec<&str> = key.splitn(4, '-').collect();
        if parts.len() < 3 {
            return false;
        }

        // 随机部分（第 3 段）必须是 32 个十六进制字符
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
