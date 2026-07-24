//! Tower Service 错误映射契约对象。

/// 把一个 Tower `Service::Error` 映射为原生响应或新的 Service 错误。
///
/// 只允许 `E -> E2` 的映射无法覆盖 Web/RPC Adapter 的真实需求：认证、授权、
/// 取消和 deadline 通常应恢复成 HTTP/gRPC 响应，而连接、协议或下游传输失败仍应
/// 作为 `Service::Error` 返回。该合同用 `Result<Response, Error>` 同时表达两条
/// 路径，不要求通用 Tower crate 依赖任何具体 Web 框架。
pub trait TowerErrorMapper: Clone + Send + Sync + 'static {
    /// 被包装 Service 原本返回的错误类型。
    type Source;
    /// 被包装 Service 与映射层共同返回的响应类型。
    type Response;
    /// 映射后仍需从 `Service::call` 返回的错误类型。
    type Error;

    /// 把原始错误恢复成响应，或转换成新的 Service 错误。
    ///
    /// # Errors
    ///
    /// 原错误不能或不应恢复成协议响应时，返回映射后的 `Service::Error`。
    fn map_error(&self, error: Self::Source) -> Result<Self::Response, Self::Error>;
}
