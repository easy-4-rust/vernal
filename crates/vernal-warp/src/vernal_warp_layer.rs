//! Warp Service 的 Vernal 组合 Layer 对象。

use std::sync::Arc;

use tower::Layer;
use vernal_context::ApplicationContext;
use vernal_tower::{RequestScopeLayer, RequestScopeService, VernalLayer, VernalService};

/// 为 `warp::service(Filter)` 组合 Context 注入与完整响应 Body Scope。
///
/// Warp 0.4 的公开 `Reply::Response` 使用私有 Body 类型，Filter 无法无损替换
/// 响应 Body。该 Layer 选择 Warp 官方公开的 Tower Service 扩展点，在服务边界
/// 泛型包装私有 Body，因此仍能保留 Frame、Trailer、背压与取消语义。
#[derive(Clone)]
pub struct VernalWarpLayer {
    context: Arc<ApplicationContext>,
}

impl VernalWarpLayer {
    /// 创建 Warp Service Layer。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

impl<S> Layer<S> for VernalWarpLayer {
    type Service = RequestScopeService<VernalService<S>>;

    fn layer(&self, inner: S) -> Self::Service {
        let context = Arc::clone(&self.context);
        RequestScopeLayer::new(Arc::clone(&context)).layer(VernalLayer::new(context).layer(inner))
    }
}
