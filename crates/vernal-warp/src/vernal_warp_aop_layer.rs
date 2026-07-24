//! Warp 严格 AOP 组合 Layer 对象。

use std::sync::Arc;

use tower::Layer;
use vernal_context::ApplicationContext;
use vernal_tower::{
    AopLayer, AopService, RequestScopeLayer, RequestScopeService, VernalLayer, VernalService,
};

use crate::{
    vernal_warp_aop_service::VernalWarpAopService, warp_route_resolver::WarpRouteResolver,
};

/// 为一个具体 Warp Filter Service 组合 Context、Scope 与严格 AOP。
///
/// Warp 不公开匹配后的模板，因此一个实例只代表调用方传入的一条完整低基数路径
/// 模式。若应用通过 `or` 组合多条路由，应先把每条需要独立操作身份的 Filter
/// 转成 Service 并分别套用该 Layer，不能传入包含用户数据的原始 URI。
#[derive(Clone)]
pub struct VernalWarpAopLayer {
    context: Arc<ApplicationContext>,
    aop_layer: AopLayer<WarpRouteResolver>,
}

impl VernalWarpAopLayer {
    /// 创建严格 AOP 组合 Layer。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>, path_pattern: impl Into<Arc<str>>) -> Self {
        Self {
            context,
            aop_layer: AopLayer::new(WarpRouteResolver::new(path_pattern)),
        }
    }
}

impl<S> Layer<S> for VernalWarpAopLayer {
    type Service =
        RequestScopeService<VernalService<VernalWarpAopService<AopService<S, WarpRouteResolver>>>>;

    fn layer(&self, inner: S) -> Self::Service {
        // 调用顺序必须是 Scope -> Context -> 错误恢复 -> AOP -> Warp Handler。
        // AOP 因而能读取前两层注入的对象；短路响应再由外层 Scope 包装 Body。
        let aop = self.aop_layer.layer(inner);
        let recovered = VernalWarpAopService::new(aop);
        let application_context = Arc::clone(&self.context);
        let context = VernalLayer::new(Arc::clone(&application_context)).layer(recovered);
        RequestScopeLayer::new(application_context).layer(context)
    }
}
