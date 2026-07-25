//! 库存预留事件记录监听器对象。

use std::{convert::Infallible, sync::Arc};

use tokio::sync::Mutex;
use vernal_context::ApplicationEventListener;

use super::InventoryReserved;

/// 把收到的事件序号写入共享测试探针。
pub struct InventoryReservedListener {
    sequences: Arc<Mutex<Vec<u64>>>,
}

impl InventoryReservedListener {
    /// 创建绑定指定共享探针的监听器。
    #[must_use]
    pub fn new(sequences: Arc<Mutex<Vec<u64>>>) -> Self {
        Self { sequences }
    }
}

impl ApplicationEventListener<InventoryReserved> for InventoryReservedListener {
    type Error = Infallible;

    fn on_event(
        &self,
        event: Arc<InventoryReserved>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        let sequences = Arc::clone(&self.sequences);
        async move {
            sequences.lock().await.push(event.sequence);
            Ok(())
        }
    }

    fn name(&self) -> &'static str {
        "test.inventory-reserved-listener"
    }
}
