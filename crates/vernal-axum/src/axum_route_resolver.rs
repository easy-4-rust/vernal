//! Axum 路由元数据解析器对象。

use axum::extract::MatchedPath;
use axum::http::Request;
use vernal_tower::TowerRouteResolver;
use vernal_web::RouteMetadata;

/// 从 Axum `MatchedPath` 生成低基数 Vernal 路由元数据。
///
/// HTTP Adapter 统一使用“路径模板 + HTTP 方法”作为 AOP 操作身份，使同一业务
/// 路由迁移到其他 Web 框架时仍可复用 Advisor。解析器不会退回原始 URI，避免
/// 用户输入进入指标标签或无限扩张调用计划。
#[derive(Clone, Copy, Debug, Default)]
pub struct AxumRouteResolver;

impl<B> TowerRouteResolver<B> for AxumRouteResolver {
    fn resolve(&self, request: &Request<B>) -> Option<RouteMetadata> {
        let path = request.extensions().get::<MatchedPath>()?.as_str();
        Some(RouteMetadata::new(
            path.to_owned(),
            request.method().as_str().to_owned(),
            path.to_owned(),
        ))
    }
}
