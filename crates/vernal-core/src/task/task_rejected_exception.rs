//! 任务拒绝异常。
//!
//! 对标 Spring `org.springframework.core.task.TaskRejectedException`。

use std::fmt;

/// 任务拒绝异常。
///
/// 对应 Java: org.springframework.core.task.TaskRejectedException
///
/// Spring 语义：任务因执行器不可用（已关闭/队列满）被拒绝时抛出。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRejectedException {
    message: String,
}

impl TaskRejectedException {
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

impl fmt::Display for TaskRejectedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "任务被拒绝: {}", self.message)
    }
}

impl std::error::Error for TaskRejectedException {}

impl From<crate::task::TaskError> for TaskRejectedException {
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
        let err = TaskRejectedException::new("executor shut down");
        assert!(err.to_string().contains("executor shut down"));
    }

    #[test]
    fn converts_from_task_error() {
        // D 类（重构安全）：与 vernal TaskError 互转
        let err = TaskRejectedException::from(TaskError::Rejected("queue full".to_string()));
        assert!(err.to_string().contains("queue full"));
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<TaskRejectedException>();
    }
}
