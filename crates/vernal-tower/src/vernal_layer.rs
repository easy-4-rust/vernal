//! `ApplicationContext` 注入 Layer 对象。

use std::sync::Arc;

use tower::Layer;
use vernal_context::ApplicationContext;

use crate::VernalService;

/// 把显式 `ApplicationContext` 注入每个 Tower Request Extension。
///
/// Layer 自身持有 Context，不读取全局变量；Axum、Tonic 等上层框架可以继续使用
/// 原生 Extension/State 提取方式。
#[derive(Clone)]
pub struct VernalLayer {
    context: Arc<ApplicationContext>,
}

impl VernalLayer {
    /// 创建 Context 注入 Layer。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

impl<S> Layer<S> for VernalLayer {
    type Service = VernalService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        VernalService::new(inner, Arc::clone(&self.context))
    }
}
