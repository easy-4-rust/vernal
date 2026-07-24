//! Tonic 路由元数据解析器对象。

use http::Request;
use tonic::GrpcMethod;
use vernal_tower::TowerRouteResolver;
use vernal_web::RouteMetadata;

/// 从 gRPC Request Extension 或规范 URI 解析稳定操作身份。
///
/// Tonic 生成的服务使用 `/{package.Service}/{Method}` 固定路径。外层 Tower Layer
/// 尚未看到解码后的 `tonic::Request` 时，解析器直接使用该规范路径；非标准或
/// 包含额外段的 URI 会拒绝解析，不把用户输入当作调用计划键。
#[derive(Clone, Copy, Debug, Default)]
pub struct TonicRouteResolver;

impl<B> TowerRouteResolver<B> for TonicRouteResolver {
    fn resolve(&self, request: &Request<B>) -> Option<RouteMetadata> {
        if let Some(route) = request.extensions().get::<RouteMetadata>() {
            return Some(route.clone());
        }
        if let Some(method) = request.extensions().get::<GrpcMethod<'static>>() {
            return Some(RouteMetadata::new(
                method.service(),
                method.method(),
                format!("/{}/{}", method.service(), method.method()),
            ));
        }

        let mut segments = request.uri().path().trim_start_matches('/').split('/');
        let service = segments.next().filter(|segment| !segment.is_empty())?;
        let method = segments.next().filter(|segment| !segment.is_empty())?;
        if segments.next().is_some() {
            return None;
        }
        Some(RouteMetadata::new(
            service.to_owned(),
            method.to_owned(),
            format!("/{service}/{method}"),
        ))
    }
}
