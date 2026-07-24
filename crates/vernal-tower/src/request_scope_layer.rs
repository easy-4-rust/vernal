//! 请求作用域 Layer 对象。

use tower::Layer;

use crate::RequestScopeService;

/// 为每个 Tower 请求建立独立 `WebRequestScope`。
#[derive(Clone, Copy, Debug, Default)]
pub struct RequestScopeLayer;

impl RequestScopeLayer {
    /// 创建请求作用域 Layer。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RequestScopeLayer {
    type Service = RequestScopeService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestScopeService::new(inner)
    }
}
