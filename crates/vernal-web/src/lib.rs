#![forbid(unsafe_code)]
#![doc = "Framework-neutral web integration contracts for Vernal."]

/// The transport family exposed by an integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    /// The contract or foundation is independent of a transport protocol.
    FrameworkNeutral,
    /// HTTP request, response, and body semantics.
    Http,
    /// RPC unary and streaming semantics.
    Rpc,
}

/// The role played by an integration crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationRole {
    /// A framework-neutral web application contract.
    ApplicationContract,
    /// A protocol-level contract.
    ProtocolContract,
    /// A lower-level service or transport foundation.
    Foundation,
    /// An HTTP application framework adapter.
    HttpFramework,
    /// An RPC framework adapter.
    RpcFramework,
}

/// Static metadata for a Vernal web integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegrationDescriptor {
    /// Vernal adapter crate.
    pub crate_name: &'static str,
    /// Upstream framework or abstraction.
    pub upstream: &'static str,
    /// Integration role.
    pub role: IntegrationRole,
    /// Primary transport family.
    pub transport: TransportKind,
    /// Current implementation maturity.
    pub status: &'static str,
}

impl IntegrationDescriptor {
    /// Creates an immutable integration descriptor.
    #[must_use]
    pub const fn new(
        crate_name: &'static str,
        upstream: &'static str,
        role: IntegrationRole,
        transport: TransportKind,
        status: &'static str,
    ) -> Self {
        Self {
            crate_name,
            upstream,
            role,
            transport,
            status,
        }
    }
}
