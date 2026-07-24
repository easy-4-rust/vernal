#![forbid(unsafe_code)]
#![doc = "Salvo integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Salvo adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-salvo",
    "salvo",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "skeleton",
);
