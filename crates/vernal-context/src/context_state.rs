//! 应用上下文状态对象。

/// `ApplicationContext` 的显式状态机。
///
/// 状态只允许由上下文公开操作推进，使并发 refresh/start/close/pause/restart 调用
/// 可以被验证，而不是依赖若干布尔标记推断当前阶段。
///
/// 与 Spring 7.0 `ConfigurableApplicationContext` 的 `isActive()` + `isClosed()`
/// 状态语义对应，并新增 `Pausing` / `Paused` 以承载 7.0 引入的
/// `pause()` / `restart()` 钩子。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ContextState {
    /// 上下文已创建，尚未解析组件。
    #[default]
    Created,
    /// 正在预热容器并初始化组件。
    Refreshing,
    /// 组件已初始化，尚未启动。
    Refreshed,
    /// 正在启动组件。
    Starting,
    /// 所有组件已启动，可以接收业务流量。
    Ready,
    /// 启动失败，正在逆序回滚。
    RollingBack,
    /// 已请求关闭，正在排空和释放。
    Draining,
    /// refresh 阶段失败但尚未完成关闭。
    Failed,
    /// 正在暂停可暂停组件（对标 Spring `LifecycleProcessor.onPause()`）。
    Pausing,
    /// 应用已暂停：可暂停组件已暂停，不可暂停组件保持原状态。
    ///
    /// `restart()` 会将状态推回 `Starting` → `Ready`。
    Paused,
    /// 资源已经释放；重复关闭保持幂等。
    Closed,
}

impl ContextState {
    /// 返回适合稳定诊断、序列化和监控标签的状态名称。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Refreshing => "refreshing",
            Self::Refreshed => "refreshed",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::RollingBack => "rolling_back",
            Self::Draining => "draining",
            Self::Failed => "failed",
            Self::Pausing => "pausing",
            Self::Paused => "paused",
            Self::Closed => "closed",
        }
    }
}
