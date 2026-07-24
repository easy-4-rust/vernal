//! Tower 请求上下文传播 Layer 对象。

use tower::Layer;

use crate::{ContextPropagationService, ExtensionRouteResolver};

/// 在请求 Scope 建立后构造并传播框架中立 `RequestContext`。
///
/// 推荐层次为：
///
/// `VernalLayer -> RequestScopeLayer -> ContextPropagationLayer -> AopLayer -> Handler`
///
/// 传播层可以脱离 AOP 单独使用，因此普通 Tower、Axum 或 Tonic Handler 也能读取
/// 相同的路由身份、owned HTTP 元数据与 Tokio 取消令牌。
#[derive(Clone)]
pub struct ContextPropagationLayer<R = ExtensionRouteResolver> {
    resolver: R,
}

impl ContextPropagationLayer<ExtensionRouteResolver> {
    /// 创建从请求 Extensions 读取 `RouteMetadata` 的传播层。
    #[must_use]
    pub const fn from_extension() -> Self {
        Self::new(ExtensionRouteResolver)
    }
}

impl<R> ContextPropagationLayer<R> {
    /// 使用指定的框架路由解析器创建传播层。
    #[must_use]
    pub const fn new(resolver: R) -> Self {
        Self { resolver }
    }
}

impl<S, R> Layer<S> for ContextPropagationLayer<R>
where
    R: Clone,
{
    type Service = ContextPropagationService<S, R>;

    fn layer(&self, inner: S) -> Self::Service {
        ContextPropagationService::new(inner, self.resolver.clone())
    }
}
