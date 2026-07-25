//! 第二个有序应用 Runner 对象。

use std::{convert::Infallible, sync::Arc};

use vernal_context::ApplicationRunner;

use super::StartupProbe;

/// 记录依赖链中的第二个 Runner。
pub struct SecondRunner {
    probe: Arc<StartupProbe>,
}

impl SecondRunner {
    /// 创建共享测试探针的 Runner。
    pub fn new(probe: Arc<StartupProbe>) -> Self {
        Self { probe }
    }
}

impl ApplicationRunner for SecondRunner {
    type Error = Infallible;

    async fn run(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        self.probe.push("runner:second");
        Ok(())
    }
}
