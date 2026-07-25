//! 应用生命周期事件合同测试支持对象。

mod application_phase_listener;
mod application_phase_probe;
mod failing_initialize_lifecycle;
mod failing_start_lifecycle;
mod ready_failure_listener;

pub use application_phase_listener::ApplicationPhaseListener;
pub use application_phase_probe::ApplicationPhaseProbe;
pub use failing_initialize_lifecycle::FailingInitializeLifecycle;
pub use failing_start_lifecycle::FailingStartLifecycle;
pub use ready_failure_listener::ReadyFailureListener;
