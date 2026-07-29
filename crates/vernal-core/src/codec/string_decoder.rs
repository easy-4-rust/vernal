//! 字符串解码器。
//!
//! 对标 Spring `org.springframework.core.codec.StringDecoder`。

use super::decoder::Decoder;

/// 字符串解码器。
///
/// 对应 Java: org.springframework.core.codec.StringDecoder
pub struct StringDecoder;

impl Decoder for StringDecoder {
    fn name(&self) -> &'static str {
        "StringDecoder"
    }

    fn supported_mime_types(&self) -> &[&'static str] {
        &["text/plain"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_string_decoder() {
        assert_eq!(StringDecoder.name(), "StringDecoder");
    }

    #[test]
    fn supports_text_plain() {
        assert!(StringDecoder.can_decode("text/plain"));
    }
}
