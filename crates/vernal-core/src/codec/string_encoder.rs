//! 字符串编码器。
//!
//! 对标 Spring `org.springframework.core.codec.StringEncoder`。

use super::encoder::Encoder;

/// 字符串编码器。
///
/// 对应 Java: org.springframework.core.codec.StringEncoder
pub struct StringEncoder;

impl Encoder for StringEncoder {
    fn name(&self) -> &'static str {
        "StringEncoder"
    }

    fn supported_mime_types(&self) -> &[&'static str] {
        &["text/plain"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_string_encoder() {
        assert_eq!(StringEncoder.name(), "StringEncoder");
    }

    #[test]
    fn supports_text_plain() {
        assert!(StringEncoder.can_encode("text/plain"));
        assert!(!StringEncoder.can_encode("application/json"));
    }
}
