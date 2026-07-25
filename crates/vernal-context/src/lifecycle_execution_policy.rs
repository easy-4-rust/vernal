//! 组件生命周期钩子执行预算对象。

use std::time::Duration;

/// 定义 `initialize`、`start`、`stop` 以及 Tokio abort 收口的最长等待时间。
///
/// Vernal 会把每个用户生命周期钩子放入独立 Tokio task：钩子在阶段预算内完成时
/// 正常返回；超过预算时先请求 Tokio abort，再等待一段受限的收口时间。该策略既
/// 防止单个组件永久占有 Context 状态机，也把同一份不可变预算作为 `IoC` 原生组件
/// 提供给需要协调自身子任务的基础设施组件。一次性
/// [`crate::ApplicationRunner`] 复用 `start` 与 abort 收口预算，但仍保留独立的
/// Runner 错误和诊断阶段。
///
/// 生命周期钩子仍必须保持异步友好并定期让出执行权。Tokio 无法强制终止在异步
/// task 内执行永久阻塞调用或无让出点死循环的代码；这类工作应由组件自行放入
/// `spawn_blocking`，并在钩子返回前完成可控的取消与收口。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifecycleExecutionPolicy {
    initialize: Duration,
    start: Duration,
    stop: Duration,
    abort_settlement: Duration,
}

impl LifecycleExecutionPolicy {
    /// 创建一份包含三个阶段预算与 abort 收口预算的不可变策略。
    #[must_use]
    pub const fn new(
        initialize_timeout: Duration,
        start_timeout: Duration,
        stop_timeout: Duration,
        abort_timeout: Duration,
    ) -> Self {
        Self {
            initialize: initialize_timeout,
            start: start_timeout,
            stop: stop_timeout,
            abort_settlement: abort_timeout,
        }
    }

    /// 返回单个组件 `initialize` 钩子的最长执行时间。
    #[must_use]
    pub const fn initialize_timeout(self) -> Duration {
        self.initialize
    }

    /// 返回单个组件 `start` 钩子或一次性 `ApplicationRunner` 的最长执行时间。
    #[must_use]
    pub const fn start_timeout(self) -> Duration {
        self.start
    }

    /// 返回单个组件 `stop` 钩子的最长执行时间。
    #[must_use]
    pub const fn stop_timeout(self) -> Duration {
        self.stop
    }

    /// 返回请求 Tokio abort 后等待任务观察器结束的最长时间。
    #[must_use]
    pub const fn abort_timeout(self) -> Duration {
        self.abort_settlement
    }
}

impl Default for LifecycleExecutionPolicy {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(30),
            Duration::from_secs(30),
            Duration::from_secs(30),
            Duration::from_secs(1),
        )
    }
}
