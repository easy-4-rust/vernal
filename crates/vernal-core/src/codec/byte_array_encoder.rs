//! 字节数组编码器。
//!
//! 对标 Spring `org.springframework.core.codec.ByteArrayEncoder`。

use super::encoder::Encoder;

/// 字节数组编码器。
///
/// 对应 Java: org.springframework.core.codec.ByteArrayEncoder
pub struct ByteArrayEncoder;

impl Encoder for ByteArrayEncoder {
    fn name(&self) -> &'static str {
        "ByteArrayEncoder"
    }

    fn supported_mime_types(&self) -> &[&'static str] {
        &["application/octet-stream"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_byte_array_encoder() {
        assert_eq!(ByteArrayEncoder.name(), "ByteArrayEncoder");
    }

    #[test]
    fn supports_octet_stream() {
        assert!(ByteArrayEncoder.can_encode("application/octet-stream"));
        assert!(!ByteArrayEncoder.can_encode("text/plain"));
    }

    #[test]
    fn mime_types_list() {
        assert_eq!(ByteArrayEncoder.supported_mime_types(), &["application/octet-stream"]);
    }
}
