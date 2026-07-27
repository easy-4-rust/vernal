//! Tonic 应用上下文拦截器对象。

use std::sync::Arc;

use tonic::{Request, Status, service::Interceptor};
use vernal_context::ApplicationContext;

/// 把显式 `ApplicationContext` 写入 Tonic 原生 Request Extensions。
///
/// 该拦截器适用于生成代码提供的 `with_interceptor` 入口。若服务端已经使用
/// [`vernal_tower::VernalLayer`]，Tonic 会把外层 HTTP Extensions 保留到
/// `tonic::Request`，此拦截器可以省略。
#[derive(Clone)]
pub struct TonicContextInterceptor {
    context: Arc<ApplicationContext>,
}

impl TonicContextInterceptor {
    /// 创建显式上下文拦截器。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

impl Interceptor for TonicContextInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request.extensions_mut().insert(Arc::clone(&self.context));
        Ok(request)
    }
}
