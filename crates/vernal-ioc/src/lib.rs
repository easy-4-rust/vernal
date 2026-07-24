#![forbid(unsafe_code)]
#![doc = "Runtime-neutral inversion of control kernel for Vernal."]

/// Returns the status of the `IoC` kernel skeleton.
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
