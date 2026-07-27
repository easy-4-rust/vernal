//! 摘要工具(feature = "digest")。
//!
//! 对标 Spring `org.springframework.util.DigestUtils`。
//!
//! 提供 SHA-256 哈希计算,输出为 hex 字符串。
//! 基于 [`sha2`] crate 0.10。
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["digest"] }
//! ```

use sha2::{Digest, Sha256};

/// 摘要工具。
///
/// 对标 Spring `DigestUtils`。
pub struct DigestUtils;

impl DigestUtils {
    /// 计算字节数据的 SHA-256 摘要(返回 hex 字符串)。
    ///
    /// 对标 Spring `DigestUtils.sha256DigestAsHex(byte[])`。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use vernal_core::util::DigestUtils;
    ///
    /// let hex = DigestUtils::sha256_hex(b"hello");
    /// assert_eq!(hex.len(), 64); // SHA-256 输出 32 字节 = 64 hex 字符
    /// ```
    #[must_use]
    pub fn sha256_hex(data: &[u8]) -> String {
        let hash = Sha256::digest(data);
        to_hex(&hash)
    }

    /// 计算字符串的 SHA-256 摘要(返回 hex 字符串)。
    ///
    /// 对标 Spring `DigestUtils.sha256DigestAsHex(String)`。
    #[must_use]
    pub fn sha256_hex_str(s: &str) -> String {
        Self::sha256_hex(s.as_bytes())
    }

    /// 计算字节数据的 SHA-256 摘要(返回原始字节)。
    #[must_use]
    pub fn sha256(data: &[u8]) -> [u8; 32] {
        let hash = Sha256::digest(data);
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        result
    }
}

/// 把字节数组转换为 hex 字符串(小写)。
fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_basic() {
        // SHA-256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        let hex = DigestUtils::sha256_hex(b"hello");
        assert_eq!(hex.len(), 64);
        assert_eq!(
            hex,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn sha256_hex_str_basic() {
        let hex = DigestUtils::sha256_hex_str("hello");
        assert_eq!(hex.len(), 64);
        assert_eq!(
            hex,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn sha256_returns_32_bytes() {
        let bytes = DigestUtils::sha256(b"hello");
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn sha256_hex_empty_input() {
        let hex = DigestUtils::sha256_hex(b"");
        assert_eq!(hex.len(), 64);
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_hex_different_inputs_different_outputs() {
        let hex1 = DigestUtils::sha256_hex(b"hello");
        let hex2 = DigestUtils::sha256_hex(b"world");
        assert_ne!(hex1, hex2);
    }

    #[test]
    fn sha256_hex_same_input_same_output() {
        let hex1 = DigestUtils::sha256_hex(b"test");
        let hex2 = DigestUtils::sha256_hex(b"test");
        assert_eq!(hex1, hex2);
    }

    #[test]
    fn to_hex_basic() {
        assert_eq!(to_hex(&[0x00, 0xff, 0xab]), "00ffab");
        assert_eq!(to_hex(&[0x01]), "01");
        assert_eq!(to_hex(&[]), "");
    }

    #[test]
    fn to_hex_lowercase() {
        let hex = to_hex(&[0xAB, 0xCD, 0xEF]);
        assert_eq!(hex, "abcdef");
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        );
    }
}
