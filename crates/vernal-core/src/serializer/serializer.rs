//! 序列化器契约。
//!
//! 对标 Spring `org.springframework.core.serializer.Serializer`。

use std::io;

/// 序列化器契约。
///
/// 对应 Java: org.springframework.core.serializer.Serializer
///
/// Spring 语义：把对象序列化为字节流（`Serializer<T>` 泛型接口）。
pub trait Serializer<T>: Send + Sync {
    /// 序列化为字节。
    ///
    /// # 错误
    ///
    /// 序列化失败时返回 [`std::io::Error`]。
    fn serialize(&self, value: &T) -> io::Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TextSerializer;

    impl Serializer<String> for TextSerializer {
        fn serialize(&self, value: &String) -> io::Result<Vec<u8>> {
            Ok(value.as_bytes().to_vec())
        }
    }

    #[test]
    fn serializer_produces_bytes() {
        // A 类（合同对齐）：对标 Spring 序列化输出
        let serializer = TextSerializer;
        assert_eq!(
            serializer.serialize(&"hello".to_string()).unwrap(),
            b"hello"
        );
    }
}
