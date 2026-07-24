#![forbid(unsafe_code)]
#![doc = "Tonic gRPC integration for Vernal."]

use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Design-stage descriptor for the Tonic adapter.
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-tonic",
    "tonic",
    IntegrationRole::RpcFramework,
    TransportKind::Rpc,
    "skeleton",
);
