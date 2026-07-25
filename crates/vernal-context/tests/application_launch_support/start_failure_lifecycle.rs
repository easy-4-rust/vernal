//! Start 失败生命周期组件对象。

use std::{io, sync::Arc};

use tokio_util::sync::CancellationToken;
use vernal_context::{Lifecycle, LifecycleFuture};
use vernal_core::BoxError;

use super::LaunchProbe;

/// initialize 成功但在 start 阶段返回业务错误的生命周期组件。
pub struct StartFailureLifecycle {
    probe: Arc<LaunchProbe>,
}

impl StartFailureLifecycle {
    /// 创建绑定共享探针的 start 失败组件。
    #[must_use]
    pub fn new(probe: Arc<LaunchProbe>) -> Self {
        Self { probe }
    }
}

impl Lifecycle for StartFailureLifecycle {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("start-failure:initialize").await;
            Ok(())
        })
    }

    fn start(&self, _cancellation: CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("start-failure:start").await;
            Err(Box::new(io::Error::other("start failure secret")) as BoxError)
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("start-failure:stop").await;
            Ok(())
        })
    }
}
