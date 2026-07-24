#![forbid(unsafe_code)]
#![doc = "Ntex integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Ntex adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-ntex",
    "ntex",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "skeleton",
);
