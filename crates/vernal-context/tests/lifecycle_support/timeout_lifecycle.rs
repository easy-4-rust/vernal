//! 生命周期 initialize/start 超时合同使用的组件对象。

use std::{
    future::pending,
    sync::atomic::{AtomicBool, Ordering},
};

use vernal_context::{Lifecycle, LifecycleFuture};

/// 按构造配置永久等待 initialize 或 start，并记录最终回滚 stop。
///
/// 永久等待使用会返回 `Poll::Pending` 的标准 Future，确保 Tokio abort 可以安全
/// 收口；测试据此验证 Context 的执行预算，而不引入真实阻塞线程或不确定 I/O。
pub struct TimeoutLifecycle {
    block_initialize: bool,
    block_start: bool,
    stopped: AtomicBool,
}

impl TimeoutLifecycle {
    /// 创建在 initialize 阶段等待到 Context 超时的组件。
    #[must_use]
    pub fn blocking_initialize() -> Self {
        Self {
            block_initialize: true,
            block_start: false,
            stopped: AtomicBool::new(false),
        }
    }

    /// 创建 initialize 成功、在 start 阶段等待到 Context 超时的组件。
    #[must_use]
    pub fn blocking_start() -> Self {
        Self {
            block_initialize: false,
            block_start: true,
            stopped: AtomicBool::new(false),
        }
    }

    /// 返回超时后的逆序回滚是否已经执行 stop。
    #[must_use]
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
}

impl Lifecycle for TimeoutLifecycle {
    /// 按配置永久等待 initialize，或立即成功。
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async {
            if self.block_initialize {
                pending::<()>().await;
            }
            Ok(())
        })
    }

    /// 按配置永久等待 start，或立即成功。
    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async {
            if self.block_start {
                pending::<()>().await;
            }
            Ok(())
        })
    }

    /// 记录 Context 已经对超时组件执行资源回滚。
    fn stop(&self) -> LifecycleFuture<'_> {
        self.stopped.store(true, Ordering::Release);
        Box::pin(async { Ok(()) })
    }
}
