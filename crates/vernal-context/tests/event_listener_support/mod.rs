//! `IoC` 托管应用事件监听器测试支持对象。

mod failing_event_listener;
mod inventory_event_module;
mod inventory_reserved;
mod inventory_reserved_listener;
mod inventory_reserving_lifecycle;

pub use failing_event_listener::FailingEventListener;
pub use inventory_event_module::InventoryEventModule;
pub use inventory_reserved::InventoryReserved;
pub use inventory_reserved_listener::InventoryReservedListener;
pub use inventory_reserving_lifecycle::InventoryReservingLifecycle;
