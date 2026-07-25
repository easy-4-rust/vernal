//! Ready 事件记录监听器对象。

use std::{convert::Infallible, sync::Arc};

use vernal_context::{ApplicationEventListener, ApplicationReadyEvent};

use super::StartupProbe;

/// 在异步 Ready 事件到达时记录最终启动事实。
pub struct ReadyRecorder {
    probe: Arc<StartupProbe>,
}

impl ReadyRecorder {
    /// 创建共享测试探针的监听器。
    pub fn new(probe: Arc<StartupProbe>) -> Self {
        Self { probe }
    }
}

impl ApplicationEventListener<ApplicationReadyEvent> for ReadyRecorder {
    type Error = Infallible;

    async fn on_event(&self, _event: Arc<ApplicationReadyEvent>) -> Result<(), Self::Error> {
        self.probe.push("event:ready");
        Ok(())
    }
}
