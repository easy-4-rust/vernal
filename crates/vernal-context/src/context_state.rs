//! 应用上下文状态对象。

/// `ApplicationContext` 的显式状态机。
///
/// 状态只允许由上下文公开操作推进，使并发 refresh/start/close 调用可以被验证，
/// 而不是依赖若干布尔标记推断当前阶段。
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
    /// 资源已经释放；重复关闭保持幂等。
    Closed,
}
