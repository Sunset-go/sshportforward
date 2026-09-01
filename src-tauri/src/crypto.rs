//! 敏感字段静态加密：AES-256-GCM。
//!
//! 主密钥为 32 字节随机值，首次启动时生成并写入应用数据目录下的
//! `encryption.key`，之后每次启动读取。密文格式：`enc1:` + base64(nonce || ciphertext)。
//!
//! 兼容性：数据库中不带 `enc1:` 前缀的值视为历史明文，解密时原样返回；
//! 启动时由一次性迁移统一重新加密（见 `lib.rs` 的 `migrate_plaintext_hosts`）。

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

/// 密文前缀，同时充当格式版本号
pub const PREFIX: &str = "enc1:";
/// AES-256 密钥长度（字节）
const KEY_LEN: usize = 32;
/// GCM 标准 nonce 长度（字节）
const NONCE_LEN: usize = 12;

static MASTER_KEY: OnceLock<Vec<u8>> = OnceLock::new();

/// 进程启动时加载或创建主密钥。必须在任何加解密调用前执行一次。
pub fn init_master_key(data_dir: &Path) -> Result<(), String> {
    let key_path = data_dir.join("encryption.key");

    let key_bytes = if key_path.exists() {
        fs::read(&key_path).map_err(|e| format!("读取密钥文件失败: {e}"))?
    } else {
        let mut k = vec![0u8; KEY_LEN];
        use rand_core::RngCore;
        OsRng.fill_bytes(&mut k);
        fs::write(&key_path, &k).map_err(|e| format!("写入密钥文件失败: {e}"))?;
        k
    };

    if key_bytes.len() != KEY_LEN {
        return Err(format!(
            "密钥文件长度错误：期望 {KEY_LEN} 字节，实际 {} 字节",
            key_bytes.len()
        ));
    }
    MASTER_KEY
        .set(key_bytes)
        .map_err(|_| "主密钥重复初始化".to_string())
}

fn cipher() -> Result<Aes256Gcm, String> {
    let key = MASTER_KEY.get().ok_or("主密钥尚未初始化")?;
    Ok(Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key)))
}

/// 判断存储值是否已是本方案的密文
pub fn is_encrypted(stored: &str) -> bool {
    stored.starts_with(PREFIX)
}

/// 加密明文，输出 `enc1:base64(nonce||ciphertext)`。空字符串原样返回（无需加密空值）。
pub fn encrypt(plain: &str) -> Result<String, String> {
    if plain.is_empty() {
        return Ok(String::new());
    }
    let c = cipher()?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ct = c
        .encrypt(&nonce, plain.as_bytes())
        .map_err(|e| format!("加密失败: {e}"))?;

    let mut packed = nonce.to_vec();
    packed.extend_from_slice(&ct);
    Ok(format!("{}{}", PREFIX, BASE64.encode(packed)))
}

/// 解密 `enc1:` 密文；不带前缀的历史明文原样返回。
pub fn decrypt(stored: &str) -> Result<String, String> {
    if stored.is_empty() || !stored.starts_with(PREFIX) {
        return Ok(stored.to_string());
    }
    let packed = BASE64
        .decode(&stored[PREFIX.len()..])
        .map_err(|e| format!("密文 base64 解码失败: {e}"))?;
    if packed.len() <= NONCE_LEN {
        return Err("密文长度非法".to_string());
    }
    let (nonce, ct) = packed.split_at(NONCE_LEN);
    let c = cipher()?;
    let plain = c
        .decrypt(Nonce::from_slice(nonce), ct)
        .map_err(|_| "解密失败：密文损坏或密钥不匹配".to_string())?;
    String::from_utf8(plain).map_err(|e| format!("解密结果不是有效 UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_key_for_test() {
        let dir = std::env::temp_dir().join(format!("sshpf-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        // 每个测试进程只初始化一次；重复调用时忽略"重复初始化"错误
        let _ = init_master_key(&dir);
    }

    #[test]
    fn roundtrip_and_unique_nonce() {
        init_key_for_test();
        let secret = "p@ssw0rd-中文-🔐";
        let enc1 = encrypt(secret).unwrap();
        let enc2 = encrypt(secret).unwrap();

        assert!(enc1.starts_with(PREFIX));
        assert_ne!(enc1, enc2, "相同明文每次加密应产生不同 nonce");
        assert_eq!(decrypt(&enc1).unwrap(), secret);
        assert_eq!(decrypt(&enc2).unwrap(), secret);
    }

    #[test]
    fn legacy_plaintext_passthrough() {
        init_key_for_test();
        assert_eq!(decrypt("").unwrap(), "");
        assert_eq!(decrypt("plain-password").unwrap(), "plain-password");
    }

    #[test]
    fn empty_stays_empty() {
        init_key_for_test();
        assert_eq!(encrypt("").unwrap(), "");
        assert_eq!(decrypt(&encrypt("").unwrap()).unwrap(), "");
    }

    #[test]
    fn tampered_ciphertext_fails() {
        init_key_for_test();
        let enc = encrypt("secret").unwrap();
        let mut tampered = enc.clone();
        tampered.pop();
        tampered.push('A');
        assert!(decrypt(&tampered).is_err());
    }
}
