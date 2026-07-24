//! 请求上下文携带契约。

use std::sync::Arc;

use crate::RequestContext;

/// 由框架原生 Request、Extension 或 State Adapter 实现的显式上下文读取合同。
pub trait ContextCarrier {
    /// 返回当前请求上下文；缺失时由 Adapter 映射为稳定基础设施错误。
    fn request_context(&self) -> Option<Arc<RequestContext>>;
}
