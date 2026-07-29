//! 字节数组解码器。
//!
//! 对标 Spring `org.springframework.core.codec.ByteArrayDecoder`。

use super::decoder::Decoder;

/// 字节数组解码器。
///
/// 对应 Java: org.springframework.core.codec.ByteArrayDecoder
pub struct ByteArrayDecoder;

impl Decoder for ByteArrayDecoder {
    fn name(&self) -> &'static str {
        "ByteArrayDecoder"
    }

    fn supported_mime_types(&self) -> &[&'static str] {
        &["application/octet-stream"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_byte_array_decoder() {
        assert_eq!(ByteArrayDecoder.name(), "ByteArrayDecoder");
    }

    #[test]
    fn supports_octet_stream() {
        assert!(ByteArrayDecoder.can_decode("application/octet-stream"));
        assert!(!ByteArrayDecoder.can_decode("text/plain"));
    }
}
