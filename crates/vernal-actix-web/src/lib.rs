#![forbid(unsafe_code)]
#![doc = "Actix Web integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Actix Web adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-actix-web",
    "actix-web",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "skeleton",
);
