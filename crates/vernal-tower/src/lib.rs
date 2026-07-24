#![forbid(unsafe_code)]
#![doc = "Tower service and layer integration foundation for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Tower foundation.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-tower",
    "tower",
    IntegrationRole::Foundation,
    TransportKind::FrameworkNeutral,
    "skeleton",
);
