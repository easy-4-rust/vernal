#![forbid(unsafe_code)]
#![doc = "Framework-neutral HTTP protocol contracts for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the HTTP protocol contract.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-http",
    "http/http-body",
    IntegrationRole::ProtocolContract,
    TransportKind::Http,
    "skeleton",
);
