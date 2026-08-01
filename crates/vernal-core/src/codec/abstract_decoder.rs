//! 抽象解码器契约。
//!
//! 对标 Spring `org.springframework.core.codec.AbstractDecoder`。

use std::io;

/// 抽象解码器契约。
///
/// 对应 Java: org.springframework.core.codec.AbstractDecoder
///
/// Spring 语义：`Decoder` 的抽象基类——提供默认解码实现与模板方法；
/// 可解码性判定（`canDecode`）在 vernal 中由 [`super::Decoder`] trait 的
/// `supported_mime_types` / `can_decode` 承担，避免方法名重复导致歧义。
pub trait AbstractDecoder: Send + Sync {
    /// 从字节解码为 `T`。
    ///
    /// # 错误
    ///
    /// 解码失败时返回 [`std::io::Error`]。
    fn decode<T>(&self, bytes: &[u8]) -> io::Result<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        text.parse::<T>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TextDecoder;

    impl AbstractDecoder for TextDecoder {}

    #[test]
    fn default_decode_parses_from_str() {
        // A 类（合同对齐）：对标 Spring 默认解码路径
        let decoder = TextDecoder;
        let value: i32 = decoder.decode(b"42").unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn invalid_bytes_return_error() {
        // C 类（错误路径）
        let decoder = TextDecoder;
        let result: io::Result<i32> = decoder.decode(b"not-a-number");
        assert!(result.is_err());
    }
}
