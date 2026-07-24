#![forbid(unsafe_code)]
#![doc = "Gotham integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Gotham adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-gotham",
    "gotham",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "skeleton",
);
