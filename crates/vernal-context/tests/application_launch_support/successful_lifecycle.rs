//! 成功启动生命周期组件对象。

use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use vernal_context::{Lifecycle, LifecycleFuture};

use super::LaunchProbe;

/// 记录 initialize、start 与 stop 顺序的成功生命周期组件。
pub struct SuccessfulLifecycle {
    probe: Arc<LaunchProbe>,
}

impl SuccessfulLifecycle {
    /// 创建绑定共享探针的成功组件。
    #[must_use]
    pub fn new(probe: Arc<LaunchProbe>) -> Self {
        Self { probe }
    }
}

impl Lifecycle for SuccessfulLifecycle {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("success:initialize").await;
            Ok(())
        })
    }

    fn start(&self, _cancellation: CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("success:start").await;
            Ok(())
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("success:stop").await;
            Ok(())
        })
    }
}
