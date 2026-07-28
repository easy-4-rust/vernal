//! Bytes 类型转换器(feature = "convert-bytes")。
//!
//! 对标 Spring 的 `ByteArrayConverter`,将字符串转换为 [`bytes::Bytes`]。
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["convert-bytes"] }
//! ```

use super::{ConversionError, Convertible};

/// Bytes 转换器。
///
/// 将 UTF-8 字符串编码为字节序列。对标 Spring 中字节数组与字符串的互转。
impl Convertible for bytes::Bytes {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        Ok(bytes::Bytes::copy_from_slice(value.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_bytes() {
        let b = bytes::Bytes::from_str_value("hello").unwrap();
        assert_eq!(b.as_ref(), b"hello");
    }

    #[test]
    fn empty_string() {
        let b = bytes::Bytes::from_str_value("").unwrap();
        assert!(b.is_empty());
    }

    #[test]
    fn unicode_to_bytes() {
        let b = bytes::Bytes::from_str_value("中文").unwrap();
        assert_eq!(b.as_ref(), "中文".as_bytes());
    }

    #[test]
    fn round_trip_via_utf8() {
        let original = "test data";
        let b = bytes::Bytes::from_str_value(original).unwrap();
        let s = std::str::from_utf8(&b).unwrap();
        assert_eq!(s, original);
    }
}
