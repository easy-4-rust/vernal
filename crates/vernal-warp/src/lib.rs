#![forbid(unsafe_code)]
#![doc = "Warp integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Warp adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-warp",
    "warp",
    IntegrationRole::HttpFramework,
    TransportKind::Http,
    "skeleton",
);
