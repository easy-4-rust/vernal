#![forbid(unsafe_code)]
#![doc = "Axum integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Axum adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-axum",
    "axum",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "skeleton",
);
