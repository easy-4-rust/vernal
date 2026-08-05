//! 嵌套运行时异常。
//!
//! 对标 Spring `org.springframework.core.NestedRuntimeException`。

use std::fmt;

use crate::nested_exception_utils::NestedExceptionUtils;

/// 嵌套运行时异常。
///
/// 对应 Java: org.springframework.core.NestedRuntimeException
///
/// Spring 语义：携带底层原因的运行时异常基类，`getRootCause` /
/// `getMostSpecificCause` 沿 `cause` 链解析。
#[derive(Debug)]
pub struct NestedRuntimeException {
    message: String,
    cause: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl NestedRuntimeException {
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
            // 沿 source 链走到底
            let mut current: &(dyn std::error::Error + 'static) = cause;
            while let Some(source) = current.source() {
                current = source;
            }
            current
        })
    }

    /// 返回最具体原因（对标 Spring `getMostSpecificCause`）。
    #[must_use]
    pub fn most_specific_cause(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.root_cause()
    }

    /// 返回是否含底层原因。
    #[must_use]
    pub fn has_cause(&self) -> bool {
        self.cause.is_some()
    }
}

impl fmt::Display for NestedRuntimeException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cause_text = self.cause.as_deref().map(ToString::to_string);
        write!(
            f,
            "{}",
            NestedExceptionUtils::build_message(&self.message, cause_text.as_deref())
        )
    }
}

impl std::error::Error for NestedRuntimeException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_deref()
            .map(|c| c as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct LeafError;

    impl fmt::Display for LeafError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "leaf")
        }
    }

    impl std::error::Error for LeafError {}

    #[test]
    fn message_only_exception() {
        // A 类（合同对齐）：对标 Spring 消息形态
        let err = NestedRuntimeException::new("operation failed");
        assert_eq!(err.to_string(), "operation failed");
        assert!(!err.has_cause());
    }

    #[test]
    fn message_includes_nested_cause() {
        // A 类（合同对齐）：对标 Spring 嵌套消息
        let err = NestedRuntimeException::with_cause("outer", Box::new(LeafError));
        assert_eq!(err.to_string(), "outer; nested exception is leaf");
        assert!(err.has_cause());
    }

    #[test]
    fn root_cause_walks_chain() {
        // C 类（错误路径）：深层原因解析
        let err = NestedRuntimeException::with_cause("outer", Box::new(LeafError));
        assert_eq!(err.root_cause().unwrap().to_string(), "leaf");
    }

    #[test]
    fn most_specific_cause_matches_root() {
        // A 类（合同对齐）：对标 Spring `getMostSpecificCause`
        let err = NestedRuntimeException::with_cause("outer", Box::new(LeafError));
        assert_eq!(err.most_specific_cause().unwrap().to_string(), "leaf");
        let plain = NestedRuntimeException::new("no cause");
        assert!(plain.most_specific_cause().is_none());
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<NestedRuntimeException>();
    }
}
