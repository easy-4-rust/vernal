//! 组件生命周期契约。

use std::any::type_name;

use tokio_util::sync::CancellationToken;

use crate::LifecycleFuture;

/// 可由应用上下文编排的异步组件。
///
/// 生命周期组件本身仍是普通 `IoC` 对象；Context 只按依赖顺序解析并调用钩子。
/// `stop` 必须能够处理“已 initialize、但 start 未完成”的回滚场景。
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
}
