//! 应用刷新与就绪事件记录监听器对象。

use std::{convert::Infallible, sync::Arc};

use vernal_context::{ApplicationEventListener, ApplicationReadyEvent, ApplicationRefreshedEvent};

use super::ApplicationPhaseProbe;

/// 使用同一个 `IoC` Singleton 分别监听刷新和就绪事件。
pub struct ApplicationPhaseListener {
    probe: Arc<ApplicationPhaseProbe>,
}

impl ApplicationPhaseListener {
    /// 创建绑定共享阶段探针的监听器。
    #[must_use]
    pub fn new(probe: Arc<ApplicationPhaseProbe>) -> Self {
        Self { probe }
    }
}

impl ApplicationEventListener<ApplicationRefreshedEvent> for ApplicationPhaseListener {
    type Error = Infallible;

    async fn on_event(&self, _event: Arc<ApplicationRefreshedEvent>) -> Result<(), Self::Error> {
        self.probe.record("refreshed");
        Ok(())
    }
}

impl ApplicationEventListener<ApplicationReadyEvent> for ApplicationPhaseListener {
    type Error = Infallible;

    async fn on_event(&self, _event: Arc<ApplicationReadyEvent>) -> Result<(), Self::Error> {
        self.probe.record("ready");
        Ok(())
    }
}
