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
