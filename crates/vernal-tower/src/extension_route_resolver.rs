//! 请求扩展路由解析器对象。

use std::sync::Arc;

use http::Request;
use vernal_web::RouteMetadata;

use crate::TowerRouteResolver;

/// 从 `http::Request::extensions` 读取路由元数据的默认解析器。
///
/// Axum、Tonic、Salvo 等上层适配器只需在进入 `AopLayer` 前写入
/// [`RouteMetadata`] 或 `Arc<RouteMetadata>`，即可复用同一套 AOP 执行逻辑。
#[derive(Clone, Copy, Debug, Default)]
pub struct ExtensionRouteResolver;

impl<B> TowerRouteResolver<B> for ExtensionRouteResolver {
    fn resolve(&self, request: &Request<B>) -> Option<RouteMetadata> {
        request
            .extensions()
            .get::<RouteMetadata>()
            .cloned()
            .or_else(|| {
                request
                    .extensions()
                    .get::<Arc<RouteMetadata>>()
                    .map(|route| route.as_ref().clone())
            })
    }
}
