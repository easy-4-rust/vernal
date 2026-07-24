#![forbid(unsafe_code)]
#![doc = "Hyper HTTP transport integration foundation for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Hyper foundation.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-hyper",
    "hyper",
    IntegrationRole::Foundation,
    TransportKind::Http,
    "skeleton",
);
