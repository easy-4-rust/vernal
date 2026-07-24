//! Tower 路由解析器对象。

use http::Request;
use vernal_web::RouteMetadata;

/// 从具体 Tower 请求中解析稳定路由元数据。
///
/// 适配器应输出低基数路径模板，不应把用户输入的原始 URI 直接作为 AOP 操作名
/// 或可观测性标签。闭包自动实现该接口，便于各 Web 框架保留自己的路由语义。
pub trait TowerRouteResolver<B>: Clone + Send + Sync + 'static {
    /// 解析当前请求对应的路由元数据。
    fn resolve(&self, request: &Request<B>) -> Option<RouteMetadata>;
}

impl<B, F> TowerRouteResolver<B> for F
where
    F: Fn(&Request<B>) -> Option<RouteMetadata> + Clone + Send + Sync + 'static,
{
    fn resolve(&self, request: &Request<B>) -> Option<RouteMetadata> {
        self(request)
    }
}
