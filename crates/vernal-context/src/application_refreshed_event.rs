//! 应用 Context 完成刷新事件值对象。

/// 表示当前 [`crate::ApplicationContext`] 已经进入 `Refreshed` 状态。
///
/// 事件由启动协调器在全部 Singleton 预热、事件监听器订阅和 Lifecycle
/// `initialize()` 成功后发布。它不携带 Context 指针、配置值或组件实例，避免
/// 形成所有权环和敏感诊断载荷；监听组件可以通过自身 `IoC` 依赖访问所需资源。
///
/// 发布意味着状态已经提交，但不等待异步监听器处理完成。监听器失败会通过
/// [`crate::ManagedTaskSupervisor`] 取消应用，而不会把已经成功的 `refresh()`
/// 伪装成同步监听回调。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ApplicationRefreshedEvent;

impl ApplicationRefreshedEvent {
    /// 创建不携带运行时数据的刷新完成事件。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
