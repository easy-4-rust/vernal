#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 应用上下文与生命周期编排。"]

mod application_context;
mod application_context_builder;
mod component_lifecycle;
mod context_error;
mod context_state;
mod event_bus;
mod lifecycle_future;
mod lifecycle_phase;

pub use application_context::ApplicationContext;
pub use application_context_builder::ApplicationContextBuilder;
pub use component_lifecycle::Lifecycle;
pub use context_error::ContextError;
pub use context_state::ContextState;
pub use event_bus::EventBus;
pub use lifecycle_future::LifecycleFuture;
pub use lifecycle_phase::LifecyclePhase;

/// 返回应用上下文当前成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
