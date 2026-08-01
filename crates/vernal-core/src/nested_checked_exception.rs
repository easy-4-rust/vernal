//! 嵌套受检异常。
//!
//! 对标 Spring `org.springframework.core.NestedCheckedException`。

use std::fmt;

use crate::nested_exception_utils::NestedExceptionUtils;

/// 嵌套受检异常。
///
/// 对应 Java: org.springframework.core.NestedCheckedException
///
/// Spring 语义：携带底层原因的受检异常基类；Rust 无受检/非受检区分
/// （所有错误经 `Result` 返回），此类型保留嵌套原因诊断语义。
#[derive(Debug)]
pub struct NestedCheckedException {
    message: String,
    cause: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl NestedCheckedException {
    /// 创建仅含消息的异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
        }
    }

    /// 创建含底层原因的异常。
    #[must_use]
    pub fn with_cause(
        message: impl Into<String>,
        cause: Box<dyn std::error::Error + Send + Sync + 'static>,
    ) -> Self {
        Self {
            message: message.into(),
            cause: Some(cause),
        }
    }

    /// 返回最底层原因（对标 Spring `getRootCause`）。
    #[must_use]
    pub fn root_cause(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_deref().map(|cause| {
            let mut current: &(dyn std::error::Error + 'static) = cause;
            while let Some(source) = current.source() {
                current = source;
            }
            current
        })
    }
}

impl fmt::Display for NestedCheckedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cause_text = self.cause.as_deref().map(ToString::to_string);
        write!(f, "{}", NestedExceptionUtils::build_message(&self.message, cause_text.as_deref()))
    }
}

impl std::error::Error for NestedCheckedException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_deref().map(|c| c as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct IoLeaf;

    impl fmt::Display for IoLeaf {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "io failure")
        }
    }

    impl std::error::Error for IoLeaf {}

    #[test]
    fn message_only() {
        // A 类（合同对齐）
        let err = NestedCheckedException::new("parse error");
        assert_eq!(err.to_string(), "parse error");
    }

    #[test]
    fn nested_message_and_root_cause() {
        // A 类（合同对齐）：对标 Spring 嵌套诊断
        let err = NestedCheckedException::with_cause("parse error", Box::new(IoLeaf));
        assert_eq!(err.to_string(), "parse error; nested exception is io failure");
        assert_eq!(err.root_cause().unwrap().to_string(), "io failure");
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<NestedCheckedException>();
    }
}
