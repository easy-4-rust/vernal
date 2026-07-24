//! refresh/start 等待者取消安全合同使用的生命周期对象。

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tokio::sync::Notify;
use vernal_context::{Lifecycle, LifecycleFuture};
use vernal_core::BoxError;

/// 在指定启动阶段阻塞并失败，用于观察调用者取消后的后台回滚。
///
/// 对象以两个布尔配置复用 initialize 与 start 场景，避免为同一合同在一个 Rust
/// 文件定义多个测试类型。无论失败发生在哪个阶段，`stop` 都会发布最终回滚证据。
pub struct CancellationSafePhaseLifecycle {
    block_initialize: bool,
    block_start: bool,
    entered: Arc<Notify>,
    release: Arc<Notify>,
    stopped: AtomicBool,
}

impl CancellationSafePhaseLifecycle {
    /// 创建将在 initialize 阶段等待并失败的组件。
    #[must_use]
    pub fn failing_initialize(entered: Arc<Notify>, release: Arc<Notify>) -> Self {
        Self {
            block_initialize: true,
            block_start: false,
            entered,
            release,
            stopped: AtomicBool::new(false),
        }
    }

    /// 创建 initialize 成功、将在 start 阶段等待并失败的组件。
    #[must_use]
    pub fn failing_start(entered: Arc<Notify>, release: Arc<Notify>) -> Self {
        Self {
            block_initialize: false,
            block_start: true,
            entered,
            release,
            stopped: AtomicBool::new(false),
        }
    }

    /// 返回后台回滚是否已经执行 stop。
    #[must_use]
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }

    /// 等待测试放行并返回不携带真实业务数据的合成错误。
    async fn wait_and_fail(&self, phase: &'static str) -> Result<(), BoxError> {
        self.entered.notify_one();
        self.release.notified().await;
        Err(Box::new(io::Error::other(phase)))
    }
}

impl Lifecycle for CancellationSafePhaseLifecycle {
    /// 按构造配置在 initialize 阶段阻塞失败。
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async {
            if self.block_initialize {
                self.wait_and_fail("synthetic initialize failure").await
            } else {
                Ok(())
            }
        })
    }

    /// 按构造配置在 start 阶段阻塞失败。
    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async {
            if self.block_start {
                self.wait_and_fail("synthetic start failure").await
            } else {
                Ok(())
            }
        })
    }

    /// 发布组件已经被后台回滚释放的证据。
    fn stop(&self) -> LifecycleFuture<'_> {
        self.stopped.store(true, Ordering::Release);
        Box::pin(async { Ok(()) })
    }
}
