//! 模拟消费方事件 Bridge 的应用模块对象。

use std::sync::Arc;

use tokio::sync::Mutex;
use vernal_beans::ComponentDefinition;
use vernal_context::{ApplicationModule, ApplicationModuleRegistrar};
use vernal_core::BoxError;

use super::{InventoryReserved, InventoryReservedListener};

/// 原子贡献监听器组件与强类型监听声明。
pub struct InventoryEventModule {
    sequences: Arc<Mutex<Vec<u64>>>,
}

impl InventoryEventModule {
    /// 创建绑定测试探针的消费方模块。
    #[must_use]
    pub fn new(sequences: Arc<Mutex<Vec<u64>>>) -> Self {
        Self { sequences }
    }
}

impl ApplicationModule for InventoryEventModule {
    fn name(&self) -> &'static str {
        "test.inventory-events"
    }

    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        registrar
            .register(ComponentDefinition::shared_arc(Arc::new(
                InventoryReservedListener::new(self.sequences),
            )))
            .event_listener::<InventoryReserved, InventoryReservedListener>();
        Ok(())
    }
}
