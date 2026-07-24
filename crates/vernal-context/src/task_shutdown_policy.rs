//! 受管 Tokio 任务停机策略对象。

use std::time::Duration;

/// 定义应用关闭时受管后台任务的两阶段等待预算。
///
/// 第一阶段只取消应用令牌并等待任务主动退出；到期后监督器调用 Tokio
/// `AbortHandle` 强制终止剩余任务，再使用第二阶段预算等待观察器收口。默认分别
/// 为 30 秒和 1 秒。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskShutdownPolicy {
    graceful_timeout: Duration,
    abort_timeout: Duration,
}

impl TaskShutdownPolicy {
    /// 创建明确的优雅等待与强制终止等待策略。
    #[must_use]
    pub const fn new(graceful_timeout: Duration, abort_timeout: Duration) -> Self {
        Self {
            graceful_timeout,
            abort_timeout,
        }
    }

    /// 返回任务收到取消后主动退出的最长等待时间。
    #[must_use]
    pub const fn graceful_timeout(self) -> Duration {
        self.graceful_timeout
    }

    /// 返回发送 Tokio abort 后观察器收口的最长等待时间。
    #[must_use]
    pub const fn abort_timeout(self) -> Duration {
        self.abort_timeout
    }
}

impl Default for TaskShutdownPolicy {
    fn default() -> Self {
        Self::new(Duration::from_secs(30), Duration::from_secs(1))
    }
}
