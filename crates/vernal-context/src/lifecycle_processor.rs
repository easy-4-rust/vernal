//! 生命周期协调器契约。

/// 生命周期协调器契约，对标 Spring 7.0
/// `org.springframework.context.LifecycleProcessor`。
///
/// `ApplicationContext` 默认实现是 `ApplicationStartupCoordinator` +
/// `ApplicationCloseCoordinator` 的组合。调用方可直接通过
/// `ApplicationContext::state()` / `ApplicationContext::is_active()` 查询当前
/// 状态，本 trait 用于框架内部一致性表达，让 `LifecycleProcessor` 的扩展接口
/// 与 Spring 完全对齐。
pub trait LifecycleProcessor: Send + Sync + 'static {
    /// 当前 Context 是否至少 refresh 过一次且未进入 Closed 终态。
    ///
    /// 对标 Spring `LifecycleProcessor#isRunning()`：
    /// - `false`：Context 处于 `Created` / `Refreshing` / `Draining` / `Failed` / `Closed`
    /// - `true`：Context 已 refresh 进入 `Refreshed` / `Starting` / `Ready` / `Pausing` / `Paused` / `RollingBack`
    ///   中的任意状态
    fn is_running(&self) -> bool;

    /// 触发 LifecycleProcessor 刷新阶段回调（对标 `LifecycleProcessor#onRefresh()`）。
    ///
    /// 默认 no-op；vernal 的 `ApplicationStartupCoordinator` 已将 refresh + start 合并到
    /// `finish_refresh` + `finish_start`，无需独立回调。
    fn on_refresh(&self) {}

    /// 触发 LifecycleProcessor 关闭阶段回调（对标 `LifecycleProcessor#onClose()`）。
    ///
    /// 默认 no-op；vernal 的 `ApplicationCloseCoordinator::finish_close` 已替代。
    fn on_close(&self) {}

    /// 触发 LifecycleProcessor 重启阶段回调（对标 `LifecycleProcessor#onRestart()`）。
    ///
    /// 默认 no-op；vernal 的 `ApplicationStartupCoordinator::finish_restart` 已替代。
    fn on_restart(&self) {}

    /// 触发 LifecycleProcessor 暂停阶段回调（对标 `LifecycleProcessor#onPause()`）。
    ///
    /// 默认 no-op；vernal 的 `ApplicationStartupCoordinator::finish_pause` 已替代。
    fn on_pause(&self) {}
}