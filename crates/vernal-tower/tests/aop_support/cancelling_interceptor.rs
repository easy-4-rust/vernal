//! 测试用协作取消拦截器对象。

use std::sync::Arc;

use vernal_aop::{Interceptor, Invocation, InvocationError, InvocationFuture, Next};
use vernal_web::WebRequestScope;

/// 取消当前请求作用域后继续推进调用链。
pub struct CancellingInterceptor;

impl Interceptor for CancellingInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            let scope = invocation
                .context()
                .get::<Arc<WebRequestScope>>()
                .await
                .ok_or_else(|| {
                    InvocationError::target(std::io::Error::other("request scope missing"))
                })?;
            scope.cancellation().cancel();
            next.run(invocation).await
        })
    }
}
