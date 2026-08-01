//! 解码异常。
//!
//! 对标 Spring `org.springframework.core.codec.DecodingException`。

use std::fmt;

use crate::codec::CodecError;

/// 解码异常。
///
/// 对应 Java: org.springframework.core.codec.DecodingException
///
/// Spring 语义：解码失败的异常（`CodecException` 子类）。
#[derive(Debug)]
pub struct DecodingException {
    message: String,
}

impl DecodingException {
    /// 创建异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for DecodingException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "解码异常: {}", self.message)
    }
}

impl std::error::Error for DecodingException {}

impl From<CodecError> for DecodingException {
    fn from(error: CodecError) -> Self {
        Self::new(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_message() {
        let err = DecodingException::new("malformed input");
        assert!(err.to_string().contains("malformed input"));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<DecodingException>();
    }
}
