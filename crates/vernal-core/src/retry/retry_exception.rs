//! 重试异常。
//!
//! 对标 Spring `org.springframework.core.retry.RetryException`。

use std::fmt;

/// 重试异常。
///
/// 对应 Java: org.springframework.core.retry.RetryException
///
/// Spring 语义：重试耗尽后抛出的终止异常。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryException {
    message: String,
}

impl RetryException {
    /// 创建异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RetryException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "重试失败: {}", self.message)
    }
}

impl std::error::Error for RetryException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_message() {
        // A 类（合同对齐）：对标 Spring 异常消息
        let err = RetryException::new("all attempts failed");
        assert!(err.to_string().contains("all attempts failed"));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<RetryException>();
    }
}
