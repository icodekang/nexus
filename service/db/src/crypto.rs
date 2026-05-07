//! Provider API Key 加密/解密模块
//!
//! 使用 AES-256-GCM 对存储在数据库中的 Provider API Key 进行加密。
//! 加密密钥从环境变量 `API_KEY_ENCRYPTION_KEY` 读取（64 位十六进制 = 32 字节）。
//!
//! # 存储格式
//! 密文格式为: base64(nonce (12 bytes) || ciphertext)
//! 与旧版纯 Base64 格式兼容：解密失败时回退到 Base64 解码。
//!
//! # 环境变量
//! - `API_KEY_ENCRYPTION_KEY`: AES-256 密钥（64 hex chars，可选）
//!   如果未设置，回退到纯 Base64 编码（不加密，向后兼容）

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use rand::RngCore;

const NONCE_SIZE: usize = 12;

/// 获取加密密钥（32 字节）
fn get_encryption_key() -> Option<[u8; 32]> {
    let hex_key = std::env::var("API_KEY_ENCRYPTION_KEY").ok()?;
    if hex_key.len() != 64 {
        tracing::warn!("API_KEY_ENCRYPTION_KEY must be 64 hex chars (32 bytes), got {} chars. Falling back to plain Base64.", hex_key.len());
        return None;
    }
    let mut key = [0u8; 32];
    match hex::decode(&hex_key) {
        Ok(bytes) if bytes.len() == 32 => {
            key.copy_from_slice(&bytes);
            Some(key)
        }
        _ => {
            tracing::warn!("API_KEY_ENCRYPTION_KEY is not valid hex. Falling back to plain Base64.");
            None
        }
    }
}

/// 简单 hex decode（避免引入额外依赖）
mod hex {
    pub fn decode(hex_str: &str) -> Result<Vec<u8>, ()> {
        if hex_str.len() % 2 != 0 {
            return Err(());
        }
        (0..hex_str.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex_str[i..i + 2], 16).map_err(|_| ())
            })
            .collect()
    }
}

/// 加密 API Key
///
/// 如果设置了 `API_KEY_ENCRYPTION_KEY` 环境变量，使用 AES-256-GCM 加密。
/// 否则回退到纯 Base64 编码（仅用于混淆，不提供加密安全性）。
///
/// # 参数
/// * `plain_key` - 明文 API Key
///
/// # 返回
/// Base64 编码的密文（或 Base64 编码的明文，如果未配置加密）
pub fn encrypt_api_key(plain_key: &str) -> String {
    let b64 = base64::engine::general_purpose::STANDARD;

    match get_encryption_key() {
        Some(key) => {
            let cipher = Aes256Gcm::new_from_slice(&key)
                .expect("AES-256-GCM key must be 32 bytes");

            let mut nonce_bytes = [0u8; NONCE_SIZE];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            match cipher.encrypt(nonce, plain_key.as_bytes()) {
                Ok(ciphertext) => {
                    // Format: nonce (12 bytes) || ciphertext
                    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
                    combined.extend_from_slice(&nonce_bytes);
                    combined.extend_from_slice(&ciphertext);
                    b64.encode(&combined)
                }
                Err(e) => {
                    tracing::error!("AES encryption failed: {:?}. Falling back to Base64.", e);
                    b64.encode(plain_key.as_bytes())
                }
            }
        }
        None => {
            // No encryption key configured — plain Base64
            b64.encode(plain_key.as_bytes())
        }
    }
}

/// 解密 API Key
///
/// 自动检测存储格式：
/// 1. 如果设置了加密密钥，尝试 AES-256-GCM 解密
/// 2. 如果 AES 解密失败（可能因为是旧格式），回退到纯 Base64 解码
/// 3. 如果未设置加密密钥，直接 Base64 解码
///
/// # 参数
/// * `encoded` - 存储的密文（Base64 编码）
///
/// # 返回
/// 明文 API Key，或错误信息
pub fn decrypt_api_key(encoded: &str) -> Result<String, String> {
    let b64 = base64::engine::general_purpose::STANDARD;

    let data = b64
        .decode(encoded)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // Try AES-256-GCM decryption if key is configured
    if let Some(key) = get_encryption_key() {
        if data.len() > NONCE_SIZE {
            let cipher = Aes256Gcm::new_from_slice(&key)
                .map_err(|_| "Invalid AES key".to_string())?;

            let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
            let nonce = Nonce::from_slice(nonce_bytes);

            match cipher.decrypt(nonce, ciphertext) {
                Ok(plaintext) => {
                    return String::from_utf8(plaintext)
                        .map_err(|e| format!("Decrypted key is not valid UTF-8: {}", e));
                }
                Err(_) => {
                    // AES decryption failed — likely old Base64-only format.
                    // Fall through to plain Base64 decode.
                }
            }
        }
    }

    // Fallback: plain Base64 (old format or no encryption configured)
    String::from_utf8(data)
        .map_err(|e| format!("API key is not valid UTF-8: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        // Without encryption key set, should use plain Base64
        let plain = "sk-test-key-12345";
        let encrypted = encrypt_api_key(plain);
        let decrypted = decrypt_api_key(&encrypted).unwrap();
        assert_eq!(decrypted, plain);
    }

    #[test]
    fn test_decrypt_old_format() {
        // Old format: plain Base64
        let b64 = base64::engine::general_purpose::STANDARD;
        let old_format = b64.encode(b"sk-old-key-67890");
        let decrypted = decrypt_api_key(&old_format).unwrap();
        assert_eq!(decrypted, "sk-old-key-67890");
    }
}
