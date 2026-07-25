//! 应用就绪事件失败监听器对象。

use std::sync::Arc;

use vernal_context::{ApplicationEventListener, ApplicationReadyEvent};

/// 模拟 Ready 后安全预热或基础设施启动失败。
pub struct ReadyFailureListener;

impl ApplicationEventListener<ApplicationReadyEvent> for ReadyFailureListener {
    type Error = std::io::Error;

    async fn on_event(&self, _event: Arc<ApplicationReadyEvent>) -> Result<(), Self::Error> {
        Err(std::io::Error::other("secret-ready-listener-response"))
    }

    fn name(&self) -> &'static str {
        "test.ready-failure-listener"
    }
}
