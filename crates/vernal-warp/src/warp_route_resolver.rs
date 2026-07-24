//! Warp 静态路由模式解析器对象。

use std::sync::Arc;

use vernal_tower::TowerRouteResolver;
use vernal_web::RouteMetadata;
use warp::http::Request;

/// 使用调用方声明的低基数路径模式构造 AOP 操作身份。
///
/// Warp 的公开 Filter/Service API 不会把最终匹配的路径模板放入 Request
/// Extensions，因此适配器不能从请求 URI 安全地恢复模板。该对象要求调用方在
/// 具体 Filter 转换成 Service 时同步声明模式，同时仍从真实请求读取 HTTP 方法。
#[derive(Clone, Debug)]
pub struct WarpRouteResolver {
    path_pattern: Arc<str>,
}

impl WarpRouteResolver {
    /// 创建显式路由模式解析器。
    #[must_use]
    pub fn new(path_pattern: impl Into<Arc<str>>) -> Self {
        Self {
            path_pattern: path_pattern.into(),
        }
    }
}

impl<B> TowerRouteResolver<B> for WarpRouteResolver {
    fn resolve(&self, request: &Request<B>) -> Option<RouteMetadata> {
        if self.path_pattern.trim().is_empty() {
            return None;
        }

        Some(RouteMetadata::new(
            Arc::clone(&self.path_pattern),
            request.method().as_str().to_owned(),
            Arc::clone(&self.path_pattern),
        ))
    }
}
