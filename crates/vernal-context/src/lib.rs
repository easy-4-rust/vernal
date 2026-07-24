#![forbid(unsafe_code)]
#![doc = "Vernal 的 Tokio-first 应用上下文与生命周期编排。"]

mod application_build_error;
mod application_context;
mod application_context_builder;
mod component_lifecycle;
mod context_error;
mod context_resources;
mod context_state;
mod diagnostic_configuration;
mod diagnostic_outcome;
mod diagnostic_phase;
mod diagnostic_state;
mod event_bus;
mod lifecycle_future;
mod lifecycle_phase;
mod scope_cleanup_policy;
mod startup_observation;
mod startup_report;
mod subsystem_status;
mod vernal_application_builder;

pub use application_build_error::ApplicationBuildError;
pub use application_context::ApplicationContext;
pub use application_context_builder::ApplicationContextBuilder;
pub use component_lifecycle::Lifecycle;
pub use context_error::ContextError;
pub use context_state::ContextState;
pub use diagnostic_outcome::DiagnosticOutcome;
pub use diagnostic_phase::DiagnosticPhase;
pub use diagnostic_state::DiagnosticState;
pub use event_bus::EventBus;
pub use lifecycle_future::LifecycleFuture;
pub use lifecycle_phase::LifecyclePhase;
pub use scope_cleanup_policy::ScopeCleanupPolicy;
pub use startup_observation::StartupObservation;
pub use startup_report::StartupReport;
pub use subsystem_status::SubsystemStatus;
pub use vernal_application_builder::VernalApplicationBuilder;

/// 返回应用上下文当前成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
