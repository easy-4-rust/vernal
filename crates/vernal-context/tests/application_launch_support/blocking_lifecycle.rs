//! 可控阻塞生命周期组件对象。

use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use vernal_context::{Lifecycle, LifecycleFuture};

use super::LaunchProbe;

/// 在 initialize 中等待测试许可，用于证明 launch 等待者取消后的后台所有权。
pub struct BlockingLifecycle {
    probe: Arc<LaunchProbe>,
}

impl BlockingLifecycle {
    /// 创建绑定共享探针的可控阻塞组件。
    #[must_use]
    pub fn new(probe: Arc<LaunchProbe>) -> Self {
        Self { probe }
    }
}

impl Lifecycle for BlockingLifecycle {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("blocking:initialize-enter").await;
            self.probe.wait_for_initialize_release().await;
            self.probe.record("blocking:initialize-exit").await;
            Ok(())
        })
    }

    fn start(&self, _cancellation: CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("blocking:start").await;
            Ok(())
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("blocking:stop").await;
            Ok(())
        })
    }
}
