#![forbid(unsafe_code)]
#![doc = "The facade crate for the Vernal framework."]

/// `AOP` kernel namespace.
pub use vernal_aop as aop;
/// Application context namespace.
pub use vernal_context as context;
/// Shared framework contracts.
pub use vernal_core as core;
/// `IoC` kernel namespace.
pub use vernal_ioc as ioc;
/// Procedural macro namespace.
pub use vernal_macros as macros;

/// Returns the current maturity status of the framework skeleton.
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
