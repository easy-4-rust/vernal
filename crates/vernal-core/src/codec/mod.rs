//! 编解码器抽象模块。
//!
//! 对标 Spring `org.springframework.core.codec`（Encoder/Decoder）。
//!
//! # 与 Spring 的对应关系
//!
//! | Spring | vernal-core |
//! |---|---|
//! | `Encoder<T>` | `Encoder` trait |
//! | `Decoder<T>` | `Decoder` trait |
//! | `CodecException` | `CodecError` enum |

/// 编解码器错误。
#[derive(Debug, Clone)]
pub enum CodecError {
    /// 编码错误
    Encode(String),
    /// 解码错误
    Decode(String),
    /// 不支持的媒体类型
    UnsupportedMediaType(String),
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Encode(msg) => write!(f, "Encode error: {msg}"),
            Self::Decode(msg) => write!(f, "Decode error: {msg}"),
            Self::UnsupportedMediaType(msg) => write!(f, "Unsupported media type: {msg}"),
        }
    }
}

impl std::error::Error for CodecError {}

/// 编码器 trait。
///
/// 对标 Spring `org.springframework.core.codec.Encoder`。
pub trait Encoder: Send + Sync {
    /// 编码类型名称。
    fn name(&self) -> &str;

    /// 支持的 MIME 类型列表。
    fn supported_mime_types(&self) -> Vec<&str>;
}

/// 解码器 trait。
///
/// 对标 Spring `org.springframework.core.codec.Decoder`。
pub trait Decoder: Send + Sync {
    /// 解码类型名称。
    fn name(&self) -> &str;

    /// 支持的 MIME 类型列表。
    fn supported_mime_types(&self) -> Vec<&str>;
}

/// 字符串编码器。
///
/// 对标 Spring `org.springframework.core.codec.StringEncoder`。
#[derive(Debug, Clone)]
pub struct StringEncoder;

impl Encoder for StringEncoder {
    fn name(&self) -> &str {
        "StringEncoder"
    }

    fn supported_mime_types(&self) -> Vec<&str> {
        vec!["text/plain"]
    }
}

/// 字符串解码器。
///
/// 对标 Spring `org.springframework.core.codec.StringDecoder`。
#[derive(Debug, Clone)]
pub struct StringDecoder;

impl Decoder for StringDecoder {
    fn name(&self) -> &str {
        "StringDecoder"
    }

    fn supported_mime_types(&self) -> Vec<&str> {
        vec!["text/plain"]
    }
}

/// 字节数组编码器。
///
/// 对标 Spring `org.springframework.core.codec.ByteArrayEncoder`。
#[derive(Debug, Clone)]
pub struct ByteArrayEncoder;

impl Encoder for ByteArrayEncoder {
    fn name(&self) -> &str {
        "ByteArrayEncoder"
    }

    fn supported_mime_types(&self) -> Vec<&str> {
        vec!["application/octet-stream"]
    }
}

/// 字节数组解码器。
///
/// 对标 Spring `org.springframework.core.codec.ByteArrayDecoder`。
#[derive(Debug, Clone)]
pub struct ByteArrayDecoder;

impl Decoder for ByteArrayDecoder {
    fn name(&self) -> &str {
        "ByteArrayDecoder"
    }

    fn supported_mime_types(&self) -> Vec<&str> {
        vec!["application/octet-stream"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_error_display() {
        let err = CodecError::Encode("invalid".to_string());
        assert!(err.to_string().contains("Encode error"));
    }

    #[test]
    fn string_encoder_name() {
        let encoder = StringEncoder;
        assert_eq!(encoder.name(), "StringEncoder");
        assert!(encoder.supported_mime_types().contains(&"text/plain"));
    }

    #[test]
    fn string_decoder_name() {
        let decoder = StringDecoder;
        assert_eq!(decoder.name(), "StringDecoder");
        assert!(decoder.supported_mime_types().contains(&"text/plain"));
    }

    #[test]
    fn byte_array_encoder_name() {
        let encoder = ByteArrayEncoder;
        assert_eq!(encoder.name(), "ByteArrayEncoder");
        assert!(encoder.supported_mime_types().contains(&"application/octet-stream"));
    }

    #[test]
    fn byte_array_decoder_name() {
        let decoder = ByteArrayDecoder;
        assert_eq!(decoder.name(), "ByteArrayDecoder");
        assert!(decoder.supported_mime_types().contains(&"application/octet-stream"));
    }
}
