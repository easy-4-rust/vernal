//! Web 集成职责对象。

/// 一个 Vernal Web crate 承担的职责。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntegrationRole {
    /// 框架中立的 Web 应用合同。
    ApplicationContract,
    /// HTTP 或 RPC 协议合同。
    ProtocolContract,
    /// 底层 Service 或 Transport 基础设施。
    Foundation,
    /// HTTP 应用框架适配器。
    HttpFramework,
    /// RPC 框架适配器。
    RpcFramework,
}
