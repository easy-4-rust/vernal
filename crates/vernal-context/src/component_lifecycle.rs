//! 组件生命周期契约。

use std::any::type_name;

use tokio_util::sync::CancellationToken;

use crate::LifecycleFuture;

/// 可由应用上下文编排的异步组件。
///
/// 生命周期组件本身仍是普通 `IoC` 对象；Context 只按依赖顺序解析并调用钩子。
/// `stop` 必须能够处理“已 initialize、但 start 未完成”的回滚场景。
///
/// 与 Spring 7.0 `Lifecycle` + `SmartLifecycle` 完整对齐：除 `initialize / start /
/// stop` 外，本 trait 还提供 `pause` 钩子和 `is_pauseable` 标志，对应 Spring
/// `SmartLifecycle#isPauseable()` 与 `LifecycleProcessor.onPause()` 语义。
/// `ApplicationContext::pause()` 只会停止声明 `is_pauseable() == true` 的组件；
/// 关闭（`close()`）停止全部组件。`restart()` 在 pause 之后重启可暂停组件。
pub trait Lifecycle: Send + Sync + 'static {
    /// 返回用于诊断的组件名称。
    fn name(&self) -> &'static str {
        type_name::<Self>()
    }

    /// 单例构造完成后的异步初始化钩子。
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async { Ok(()) })
    }

    /// 启动组件。
    ///
    /// 组件应监听传入的取消令牌，并让自行派生的 Tokio task 在关闭时退出。
    fn start(&self, _cancellation: CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async { Ok(()) })
    }

    /// 释放初始化或启动阶段获得的资源。
    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async { Ok(()) })
    }

    /// 暂停组件 —— 与 Spring `SmartLifecycle#isPauseable()` 钩子语义一致。
    ///
    /// `ApplicationContext::pause()` 在到达 `Ready` 之后调用本钩子：可暂停组件
    /// （即 [`Self::is_pauseable`] 返回 `true`）按依赖逆序暂停，其余组件保持
    /// 当前状态。`ApplicationContext::close()` 与失败回滚仍然调用 `stop`，不
    /// 重复调用 `pause`；`restart()` 调用 `start` 重新激活可暂停组件。
    ///
    /// 默认实现为空操作，与 Spring 7.0 的 `default boolean isPauseable() { return true; }`
    /// 相同 —— 组件可选择覆盖本钩子完成“暂停但不释放资源”的工作，例如暂停
    /// 消息消费、停止接受新请求或冻结缓存。
    fn pause(&self) -> LifecycleFuture<'_> {
        Box::pin(async { Ok(()) })
    }

    /// 是否参与 Context `pause()` / `restart()` 序列。
    ///
    /// 对标 Spring 7.0 `SmartLifecycle#isPauseable()`。返回 `false` 时，
    /// `ApplicationContext::pause()` 跳过该组件；`restart()` 同样跳过。返回
    /// `true` 但组件未覆盖 [`Self::pause`] 时仍能正常进入暂停状态，只是没有
    /// 业务侧工作。
    ///
    /// 默认 `true`，与 Spring 7.0 保持一致：所有 `SmartLifecycle` 组件默认参与
    /// 暂停/重启序列。需要跳过暂停的组件显式返回 `false`。
    fn is_pauseable(&self) -> bool {
        true
    }
}
