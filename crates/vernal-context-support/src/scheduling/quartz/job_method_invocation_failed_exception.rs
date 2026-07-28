//! 任务方法调用失败异常 — 对标 `JobMethodInvocationFailedException`。

/// 任务方法调用失败异常。
///
/// 对标 Spring 的 `JobMethodInvocationFailedException`，在任务方法调用失败时抛出。
#[derive(Debug, thiserror::Error)]
#[error("任务方法调用失败：{0}")]
pub struct JobMethodInvocationFailedException(pub String);

impl JobMethodInvocationFailedException {
    /// 创建异常。
    pub fn new(message: String) -> Self {
        Self(message)
    }
}
