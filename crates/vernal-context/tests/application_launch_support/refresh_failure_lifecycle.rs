//! Refresh 失败生命周期组件对象。

use std::{io, sync::Arc};

use vernal_context::{Lifecycle, LifecycleFuture};
use vernal_core::BoxError;

use super::LaunchProbe;

/// 在 initialize 阶段返回业务错误的生命周期组件。
pub struct RefreshFailureLifecycle {
    probe: Arc<LaunchProbe>,
}

impl RefreshFailureLifecycle {
    /// 创建绑定共享探针的 refresh 失败组件。
    #[must_use]
    pub fn new(probe: Arc<LaunchProbe>) -> Self {
        Self { probe }
    }
}

impl Lifecycle for RefreshFailureLifecycle {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.probe.record("refresh-failure:initialize").await;
            Err(Box::new(io::Error::other("refresh failure secret")) as BoxError)
        })
    }
}
