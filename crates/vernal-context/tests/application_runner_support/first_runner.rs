//! 第一个有序应用 Runner 对象。

use std::{convert::Infallible, sync::Arc};

use vernal_context::ApplicationRunner;

use super::StartupProbe;

/// 记录依赖链中的第一个 Runner。
pub struct FirstRunner {
    probe: Arc<StartupProbe>,
}

impl FirstRunner {
    /// 创建共享测试探针的 Runner。
    pub fn new(probe: Arc<StartupProbe>) -> Self {
        Self { probe }
    }
}

impl ApplicationRunner for FirstRunner {
    type Error = Infallible;

    async fn run(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        self.probe.push("runner:first");
        Ok(())
    }
}
