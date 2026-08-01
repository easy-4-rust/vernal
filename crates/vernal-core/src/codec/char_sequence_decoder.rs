//! 字符序列解码器。
//!
//! 对标 Spring `org.springframework.core.codec.CharSequenceDecoder`。

use super::{AbstractCharSequenceDecoder, AbstractDecoder, Decoder};

/// 字符序列解码器。
///
/// 对应 Java: org.springframework.core.codec.CharSequenceDecoder
///
/// Spring 语义：把数据缓冲解码为字符序列的 UTF-8 解码器，
/// 仅接受文本类 MIME 类型（`text/*` 与 `application/*+json` 等）。
pub struct CharSequenceDecoder {
    /// 支持的 MIME 类型。
    mime_types: Vec<&'static str>,
}

impl CharSequenceDecoder {
    /// 创建默认解码器（接受全部文本类型）。
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

    /// 创建限定 MIME 类型的解码器。
    #[must_use]
    pub fn with_mime_types(mime_types: Vec<&'static str>) -> Self {
        Self { mime_types }
    }
}

impl Default for CharSequenceDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// 继承链标记：对标 Spring `CharSequenceDecoder extends AbstractCharSequenceDecoder`。
impl AbstractDecoder for CharSequenceDecoder {}

impl AbstractCharSequenceDecoder for CharSequenceDecoder {}

impl Decoder for CharSequenceDecoder {
    fn name(&self) -> &'static str {
        "charSequenceDecoder"
    }

    fn supported_mime_types(&self) -> &[&str] {
        &self.mime_types
    }

    fn can_decode(&self, mime_type: &str) -> bool {
        self.mime_types.contains(&mime_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_utf8_text() {
        // A 类（合同对齐）：对标 Spring 字符序列解码
        let decoder = CharSequenceDecoder::new();
        let text = decoder.decode_to_string("你好, vernal".as_bytes()).unwrap();
        assert_eq!(text, "你好, vernal");
        assert_eq!(decoder.name(), "charSequenceDecoder");
    }

    #[test]
    fn mime_type_gate() {
        // B 类（边界行为）：文本类型放行、二进制类型拒绝
        let decoder = CharSequenceDecoder::new();
        assert!(decoder.can_decode("text/plain"));
        assert!(!decoder.can_decode("application/octet-stream"));
    }

    #[test]
    fn rejects_invalid_utf8() {
        // C 类（错误路径）
        let decoder = CharSequenceDecoder::new();
        assert!(decoder.decode_to_string(&[0xFF, 0xFE, 0x00]).is_err());
    }
}
