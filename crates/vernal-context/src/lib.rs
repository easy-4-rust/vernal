#![forbid(unsafe_code)]
#![doc = "Application context and lifecycle composition for Vernal."]

/// Returns the status of the application context skeleton.
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
