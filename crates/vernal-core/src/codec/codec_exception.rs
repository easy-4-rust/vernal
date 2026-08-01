//! 编解码异常。
//!
//! 对标 Spring `org.springframework.core.codec.CodecException`。

use std::fmt;

use crate::codec::CodecError;

/// 编解码异常。
///
/// 对应 Java: org.springframework.core.codec.CodecException
///
/// Spring 语义：编解码失败的运行时异常基类；vernal 的编解码错误统一由
/// [`CodecError`] 表达，本类型提供 Spring 命名兼容形态。
#[derive(Debug)]
pub struct CodecException {
    message: String,
}

impl CodecException {
    /// 创建异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CodecException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "编解码异常: {}", self.message)
    }
}

impl std::error::Error for CodecException {}

impl From<CodecError> for CodecException {
    fn from(error: CodecError) -> Self {
        Self::new(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_message() {
        // A 类（合同对齐）：对标 Spring 异常消息
        let err = CodecException::new("unsupported format");
        assert!(err.to_string().contains("unsupported format"));
    }

    #[test]
    fn converts_from_codec_error() {
        // D 类（重构安全）：与 vernal CodecError 互转
        let err = CodecException::from(CodecError::Encode("bad".to_string()));
        assert!(err.to_string().contains("bad"));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<CodecException>();
    }
}
