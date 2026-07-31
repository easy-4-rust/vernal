//! 编解码错误。
//!
//! 对标 Spring `org.springframework.core.codec.CodecException`。

/// 编解码错误。
///
/// 对应 Java: org.springframework.core.codec.CodecException
#[derive(Debug, Clone)]
pub enum CodecError {
    /// 编码错误：对标 `EncoderException`
    Encode(String),
    /// 解码错误：对标 `DecoderException`
    Decode(String),
    /// 不支持的媒体类型
    UnsupportedMediaType(String),
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Encode(msg) => write!(f, "编码失败: {msg}"),
            Self::Decode(msg) => write!(f, "解码失败: {msg}"),
            Self::UnsupportedMediaType(msg) => write!(f, "不支持的媒体类型: {msg}"),
        }
    }
}

impl std::error::Error for CodecError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_displays_message() {
        // 对标 Spring EncoderException: Display 包含消息
        let err = CodecError::Encode("bad bytes".to_string());
        assert_eq!(format!("{err}"), "编码失败: bad bytes");
    }

    #[test]
    fn decode_displays_message() {
        // 对标 Spring DecoderException
        let err = CodecError::Decode("malformed json".to_string());
        assert_eq!(format!("{err}"), "解码失败: malformed json");
    }

    #[test]
    fn unsupported_media_type_displays_message() {
        let err = CodecError::UnsupportedMediaType("application/x-unknown".to_string());
        assert_eq!(format!("{err}"), "不支持的媒体类型: application/x-unknown");
    }

    #[test]
    fn error_trait_source_returns_none() {
        // 对标 std::error::Error: source() 默认返回 None
        let err = CodecError::Encode("x".to_string());
        use std::error::Error as _;
        assert!(err.source().is_none());
    }
}
