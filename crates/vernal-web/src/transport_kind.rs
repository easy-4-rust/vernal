//! Web 集成传输类型对象。

/// 集成对外暴露的主要传输族。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportKind {
    /// 与具体传输协议无关的应用层合同。
    FrameworkNeutral,
    /// HTTP 请求、响应和 Body 语义。
    Http,
    /// RPC Unary 与 Streaming 语义。
    Rpc,
}
