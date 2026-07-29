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
