//! Runner 测试生命周期组件对象。

use std::sync::Arc;

use vernal_context::{Lifecycle, LifecycleFuture};

use super::StartupProbe;

/// 记录生命周期 start 与 stop 边界。
pub struct StartedLifecycle {
    probe: Arc<StartupProbe>,
}

impl StartedLifecycle {
    /// 创建共享测试探针的生命周期组件。
    pub fn new(probe: Arc<StartupProbe>) -> Self {
        Self { probe }
    }
}

impl Lifecycle for StartedLifecycle {
    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async {
            self.probe.push("lifecycle:start");
            Ok(())
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async {
            self.probe.push("lifecycle:stop");
            Ok(())
        })
    }
}
