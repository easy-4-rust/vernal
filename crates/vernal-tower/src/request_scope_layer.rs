//! 请求作用域 Layer 对象。

use std::sync::Arc;

use tower::Layer;
use vernal_context::ApplicationContext;

use crate::RequestScopeService;

/// 为每个 Tower 请求建立绑定应用组件图的独立 `WebRequestScope`。
///
/// Layer 显式持有应用上下文，不依赖其他 Layer 先把 Context 写入 Request
/// Extensions，因此组合顺序不会决定请求作用域能否解析 `IoC` 组件。
#[derive(Clone)]
pub struct RequestScopeLayer {
    context: Arc<ApplicationContext>,
}

impl RequestScopeLayer {
    /// 创建绑定指定应用上下文的请求作用域 Layer。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

impl<S> Layer<S> for RequestScopeLayer {
    type Service = RequestScopeService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestScopeService::new(inner, Arc::clone(&self.context))
    }
}
