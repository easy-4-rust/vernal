//! 任务超时异常。
//!
//! 对标 Spring `org.springframework.core.task.TaskTimeoutException`。

use std::fmt;

/// 任务超时异常。
///
/// 对应 Java: org.springframework.core.task.TaskTimeoutException
///
/// Spring 语义：任务在限定时间内未被执行（如队列等待超时）时抛出。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskTimeoutException {
    message: String,
}

impl TaskTimeoutException {
    /// 创建异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// 返回消息。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for TaskTimeoutException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "任务超时: {}", self.message)
    }
}

impl std::error::Error for TaskTimeoutException {}

impl From<crate::task::TaskError> for TaskTimeoutException {
    fn from(error: crate::task::TaskError) -> Self {
        Self::new(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskError;

    #[test]
    fn displays_message() {
        // A 类（合同对齐）：对标 Spring 异常消息
        let err = TaskTimeoutException::new("timed out after 5s");
        assert!(err.to_string().contains("timed out after 5s"));
    }

    #[test]
    fn converts_from_task_error() {
        // D 类（重构安全）：与 vernal TaskError 互转
        let err = TaskTimeoutException::from(TaskError::Timeout("5s".to_string()));
        assert!(err.to_string().contains("5s"));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<TaskTimeoutException>();
    }
}
