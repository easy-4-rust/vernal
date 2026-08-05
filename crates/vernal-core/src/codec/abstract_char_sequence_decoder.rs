//! 抽象字符序列解码器契约。
//!
//! 对标 Spring `org.springframework.core.codec.AbstractCharSequenceDecoder`。

use std::io;

use super::AbstractDecoder;

/// 抽象字符序列解码器契约。
///
/// 对应 Java: org.springframework.core.codec.AbstractCharSequenceDecoder
///
/// Spring 语义：面向字符序列的 `AbstractDecoder` 子类——把字节解码为
/// UTF-8 文本后交给子类处理。
pub trait AbstractCharSequenceDecoder: AbstractDecoder {
    /// 把字节解码为字符串（默认实现：UTF-8 严格解码）。
    ///
    /// # 错误
    ///
    /// 非 UTF-8 输入返回 [`std::io::Error`]。
    fn decode_to_string(&self, bytes: &[u8]) -> io::Result<String> {
        String::from_utf8(bytes.to_vec()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Utf8Decoder;

    impl AbstractDecoder for Utf8Decoder {}
    impl AbstractCharSequenceDecoder for Utf8Decoder {}

    #[test]
    fn decodes_utf8_text() {
        // A 类（合同对齐）：对标 Spring 字符序列解码
        let decoder = Utf8Decoder;
        assert_eq!(decoder.decode_to_string("你好".as_bytes()).unwrap(), "你好");
    }

    #[test]
    fn invalid_utf8_returns_error() {
        // C 类（错误路径）
        let decoder = Utf8Decoder;
        assert!(decoder.decode_to_string(&[0xFF, 0x00]).is_err());
    }
}
