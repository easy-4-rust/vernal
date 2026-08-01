//! 编码异常。
//!
//! 对标 Spring `org.springframework.core.codec.EncodingException`。

use std::fmt;

use crate::codec::CodecError;

/// 编码异常。
///
/// 对应 Java: org.springframework.core.codec.EncodingException
///
/// Spring 语义：编码失败的异常（`CodecException` 子类）。
#[derive(Debug)]
pub struct EncodingException {
    message: String,
}

impl EncodingException {
    /// 创建异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for EncodingException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "编码异常: {}", self.message)
    }
}

impl std::error::Error for EncodingException {}

impl From<CodecError> for EncodingException {
    fn from(error: CodecError) -> Self {
        Self::new(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_message() {
        let err = EncodingException::new("cannot encode");
        assert!(err.to_string().contains("cannot encode"));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<EncodingException>();
    }
}
