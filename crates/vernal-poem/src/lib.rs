#![forbid(unsafe_code)]
#![doc = "Poem integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Poem adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-poem",
    "poem",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "skeleton",
);
