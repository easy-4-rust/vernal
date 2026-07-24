//! 应用作用域清理策略对象。

use std::time::Duration;

/// 定义应用拥有的异步 Scope 最多等待多久才向调用方报告超时。
///
/// 超时只结束当前等待者，不会取消已经启动的后台清理任务；关闭钩子仍会继续
/// 逆序执行，最终释放缓存并将 Scope 转换为 Closed。默认上限为 30 秒，避免
/// Web/RPC 连接因失控的资源钩子永久占用，同时给数据库事务、消息确认等正常
/// 异步释放留下明确预算。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopeCleanupPolicy {
    timeout: Option<Duration>,
}

impl ScopeCleanupPolicy {
    /// 创建带明确等待上限的清理策略。
    #[must_use]
    pub const fn bounded(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
        }
    }

    /// 创建不限制等待时间的兼容策略。
    ///
    /// 该模式适合离线任务或调用方已经在外层实施 deadline 的场景；面向网络请求
    /// 的应用通常应保留默认有界策略。
    #[must_use]
    pub const fn unbounded() -> Self {
        Self { timeout: None }
    }

    /// 返回清理等待上限；`None` 表示由调用方无限等待。
    #[must_use]
    pub const fn timeout(self) -> Option<Duration> {
        self.timeout
    }
}

impl Default for ScopeCleanupPolicy {
    fn default() -> Self {
        Self::bounded(Duration::from_secs(30))
    }
}
