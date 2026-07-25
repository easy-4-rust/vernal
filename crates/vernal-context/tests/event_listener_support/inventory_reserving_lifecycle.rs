//! 在 initialize 阶段发布库存事件的生命周期组件对象。

use std::sync::Arc;

use vernal_context::{EventBus, Lifecycle, LifecycleFuture};

use super::InventoryReserved;

/// 验证监听订阅早于普通生命周期 initialize 建立。
pub struct InventoryReservingLifecycle {
    events: Arc<EventBus>,
}

impl InventoryReservingLifecycle {
    /// 创建使用 Context 内同一事件总线的发布组件。
    #[must_use]
    pub fn new(events: Arc<EventBus>) -> Self {
        Self { events }
    }
}

impl Lifecycle for InventoryReservingLifecycle {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let delivered = self
                .events
                .publish(InventoryReserved { sequence: 41 })
                .await;
            if delivered == 1 {
                Ok(())
            } else {
                Err(Box::new(std::io::Error::other(format!(
                    "expected one initialized event listener, got {delivered}"
                ))) as vernal_core::BoxError)
            }
        })
    }
}
