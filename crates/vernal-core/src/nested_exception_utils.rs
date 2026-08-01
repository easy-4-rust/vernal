//! 嵌套异常工具。
//!
//! 对标 Spring `org.springframework.core.NestedExceptionUtils`。

/// 嵌套异常工具（静态辅助函数）。
///
/// 对应 Java: org.springframework.core.NestedExceptionUtils
pub struct NestedExceptionUtils;

impl NestedExceptionUtils {
    /// 构建带嵌套消息的诊断文本。
    ///
    /// 对标 Spring `buildMessage(String, Throwable)`——仅在存在嵌套原因时
    /// 附加 "; nested exception is ..." 后缀。
    #[must_use]
    pub fn build_message(message: &str, cause_message: Option<&str>) -> String {
        match cause_message {
            Some(cause) if !cause.is_empty() => {
                format!("{message}; nested exception is {cause}")
            }
            _ => message.to_string(),
        }
    }

    /// 返回错误链中最底层原因（对标 Spring `getRootCause`）。
    #[must_use]
    pub fn root_cause<'a>(error: &'a (dyn std::error::Error + 'a)) -> &'a (dyn std::error::Error + 'a) {
        let mut current = error;
        while let Some(source) = current.source() {
            current = source;
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct LeafError;

    impl std::fmt::Display for LeafError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "leaf failure")
        }
    }

    impl std::error::Error for LeafError {}

    #[derive(Debug)]
    struct WrapperError {
        source: LeafError,
    }

    impl std::fmt::Display for WrapperError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "wrapper failure")
        }
    }

    impl std::error::Error for WrapperError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.source)
        }
    }

    #[test]
    fn appends_nested_suffix_only_with_cause() {
        // A 类（合同对齐）：对标 Spring `buildMessage`
        assert_eq!(
            NestedExceptionUtils::build_message("failed", Some("root cause")),
            "failed; nested exception is root cause"
        );
        assert_eq!(NestedExceptionUtils::build_message("failed", None), "failed");
    }

    #[test]
    fn walks_to_deepest_cause() {
        // A 类（合同对齐）：对标 Spring `getRootCause`
        let error = WrapperError { source: LeafError };
        let root = NestedExceptionUtils::root_cause(&error);
        assert_eq!(root.to_string(), "leaf failure");
    }
}
