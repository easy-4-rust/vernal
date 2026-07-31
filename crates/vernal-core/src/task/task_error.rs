//! 任务执行错误。
//!
//! 对标 Spring `org.springframework.core.task.TaskRejectedException` /
//! `org.springframework.core.task.TaskTimeoutException`。

/// 任务执行错误。
///
/// 对应 Java: org.springframework.core.task.TaskRejectedException
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskError {
    /// 任务被拒绝（对标 `TaskRejectedException`）。
    Rejected(String),
    /// 任务执行超时（对标 `TaskTimeoutException`）。
    Timeout(String),
    /// 任务执行失败。
    ExecutionFailed(String),
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected(msg) => write!(f, "任务被拒绝: {msg}"),
            Self::Timeout(msg) => write!(f, "任务超时: {msg}"),
            Self::ExecutionFailed(msg) => write!(f, "任务执行失败: {msg}"),
        }
    }
}

impl std::error::Error for TaskError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// 对标 Spring `TaskRejectedException` 消息包含拒绝原因
    #[test]
    fn rejected_display_includes_message() {
        let err = TaskError::Rejected("queue full".to_string());
        assert!(err.to_string().contains("任务被拒绝"));
        assert!(err.to_string().contains("queue full"));
    }

    /// 对标 Spring `TaskTimeoutException` 消息包含超时详情
    #[test]
    fn timeout_display_includes_message() {
        let err = TaskError::Timeout("30s elapsed".to_string());
        assert!(err.to_string().contains("任务超时"));
        assert!(err.to_string().contains("30s elapsed"));
    }

    /// 自定义执行失败变体
    #[test]
    fn execution_failed_display_includes_message() {
        let err = TaskError::ExecutionFailed("panic in worker".to_string());
        assert!(err.to_string().contains("任务执行失败"));
        assert!(err.to_string().contains("panic in worker"));
    }

    /// 错误实现 std::error::Error（对标 Spring 异常链）
    #[test]
    fn task_error_implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<TaskError>();
    }

    /// PartialEq 应能区分三种变体
    #[test]
    fn task_error_partial_eq_distinguishes_variants() {
        let rejected = TaskError::Rejected("a".to_string());
        let timeout = TaskError::Timeout("a".to_string());
        let failed = TaskError::ExecutionFailed("a".to_string());
        assert_ne!(rejected, timeout);
        assert_ne!(rejected, failed);
        assert_ne!(timeout, failed);
        // 相同变体 + 相同消息应相等
        assert_eq!(rejected, TaskError::Rejected("a".to_string()));
    }
}
