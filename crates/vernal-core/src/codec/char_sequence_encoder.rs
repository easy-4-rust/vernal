//! 字符序列编码器。
//!
//! 对标 Spring `org.springframework.core.codec.CharSequenceEncoder`。

use std::io;

use super::{AbstractEncoder, AbstractSingleValueEncoder, Encoder};

/// 字符序列编码器。
///
/// 对应 Java: org.springframework.core.codec.CharSequenceEncoder
///
/// Spring 语义：把 `CharSequence` 编码为 UTF-8 字节流的编码器，
/// 默认接受文本类 MIME 类型。
pub struct CharSequenceEncoder {
    /// 支持的 MIME 类型。
    mime_types: Vec<&'static str>,
}

impl CharSequenceEncoder {
    /// 创建默认编码器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            mime_types: vec![
                "text/plain",
                "text/html",
                "text/xml",
                "application/json",
                "application/xml",
            ],
        }
    }

    /// 创建限定 MIME 类型的编码器。
    #[must_use]
    pub fn with_mime_types(mime_types: Vec<&'static str>) -> Self {
        Self { mime_types }
    }
}

impl Default for CharSequenceEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// 继承链标记：对标 Spring `CharSequenceEncoder extends AbstractEncoder`。
impl AbstractEncoder for CharSequenceEncoder {}

impl AbstractSingleValueEncoder for CharSequenceEncoder {
    fn encode_value<T: std::fmt::Display>(&self, value: &T) -> io::Result<Vec<u8>> {
        self.encode(value)
    }
}

impl Encoder for CharSequenceEncoder {
    fn name(&self) -> &'static str {
        "charSequenceEncoder"
    }

    fn supported_mime_types(&self) -> &[&str] {
        &self.mime_types
    }

    fn can_encode(&self, mime_type: &str) -> bool {
        self.mime_types.contains(&mime_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_string_to_utf8() {
        // A 类（合同对齐）：对标 Spring 字符序列编码
        let encoder = CharSequenceEncoder::new();
        let bytes = encoder.encode(&"你好").unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), "你好");
        assert_eq!(encoder.name(), "charSequenceEncoder");
    }

    #[test]
    fn mime_type_gate() {
        // B 类（边界行为）：文本类型放行、二进制类型拒绝
        let encoder = CharSequenceEncoder::new();
        assert!(encoder.can_encode("application/json"));
        assert!(!encoder.can_encode("image/png"));
    }
}
