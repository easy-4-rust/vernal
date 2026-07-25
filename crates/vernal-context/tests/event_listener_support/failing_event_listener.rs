//! 返回脱敏错误的测试事件监听器对象。

use std::sync::Arc;

use vernal_context::ApplicationEventListener;

use super::InventoryReserved;

/// 模拟安全审计或领域事件落库失败。
pub struct FailingEventListener;

impl ApplicationEventListener<InventoryReserved> for FailingEventListener {
    type Error = std::io::Error;

    async fn on_event(&self, _event: Arc<InventoryReserved>) -> Result<(), Self::Error> {
        Err(std::io::Error::other("secret-listener-storage-response"))
    }

    fn name(&self) -> &'static str {
        "test.failing-event-listener"
    }
}
