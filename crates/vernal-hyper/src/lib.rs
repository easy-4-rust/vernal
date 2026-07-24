#![forbid(unsafe_code)]
#![doc = "Vernal 与 Hyper 1.x 之间的流式传输桥接。"]

mod hyper_bridge;

pub use hyper_bridge::HyperBridge;
use vernal_web::{IntegrationDescriptor, IntegrationRole, TransportKind};

/// Hyper 传输桥接的静态集成描述。
pub const INTEGRATION: IntegrationDescriptor = IntegrationDescriptor::new(
    "vernal-hyper",
    "hyper",
    IntegrationRole::Foundation,
    TransportKind::Http,
    "phase-4-foundation",
);
