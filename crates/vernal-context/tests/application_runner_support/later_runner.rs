//! 失败后不应执行的后续 Runner 对象。

use std::{convert::Infallible, sync::Arc};

use vernal_context::ApplicationRunner;

use super::StartupProbe;

/// 用于证明 Runner 链遇错即停。
pub struct LaterRunner {
    probe: Arc<StartupProbe>,
}

impl LaterRunner {
    /// 创建共享测试探针的 Runner。
    pub fn new(probe: Arc<StartupProbe>) -> Self {
        Self { probe }
    }
}

impl ApplicationRunner for LaterRunner {
    type Error = Infallible;

    async fn run(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        self.probe.push("runner:later");
        Ok(())
    }
}
