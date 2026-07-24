#![forbid(unsafe_code)]
#![doc = "Runtime-neutral aspect and interception kernel for Vernal."]

/// Returns the status of the `AOP` kernel skeleton.
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
