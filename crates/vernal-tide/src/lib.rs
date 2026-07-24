#![forbid(unsafe_code)]
#![doc = "Tide integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Tide adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-tide",
    "tide",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "skeleton",
);
