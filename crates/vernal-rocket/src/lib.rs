#![forbid(unsafe_code)]
#![doc = "Rocket integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Rocket adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-rocket",
    "rocket",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "skeleton",
);
