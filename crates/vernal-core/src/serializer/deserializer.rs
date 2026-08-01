//! 反序列化器契约。
//!
//! 对标 Spring `org.springframework.core.serializer.Deserializer`。

use std::io;

/// 反序列化器契约。
///
/// 对应 Java: org.springframework.core.serializer.Deserializer
///
/// Spring 语义：把字节流反序列化为对象（`Deserializer<T>` 泛型接口）。
pub trait Deserializer<T>: Send + Sync {
    /// 从字节反序列化。
    ///
    /// # 错误
    ///
    /// 反序列化失败时返回 [`std::io::Error`]。
    fn deserialize(&self, bytes: &[u8]) -> io::Result<T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Utf8Deserializer;

    impl Deserializer<String> for Utf8Deserializer {
        fn deserialize(&self, bytes: &[u8]) -> io::Result<String> {
            String::from_utf8(bytes.to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        }
    }

    #[test]
    fn deserializer_reads_bytes() {
        // A 类（合同对齐）：对标 Spring 反序列化
        let deserializer = Utf8Deserializer;
        assert_eq!(deserializer.deserialize(b"hello").unwrap(), "hello");
    }

    #[test]
    fn invalid_bytes_return_error() {
        // C 类（错误路径）
        let deserializer = Utf8Deserializer;
        assert!(deserializer.deserialize(&[0xFF, 0xFE]).is_err());
    }
}
