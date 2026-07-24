//! 测试用 Gotham 原生响应短路拦截器对象。

use std::sync::Arc;

use gotham::handler::IntoBody;
use http::{Response, StatusCode};
use vernal_aop::{Interceptor, Invocation, InvocationFuture, InvocationValue, Next};
use vernal_gotham::GothamResponse;

/// 不推进 Handler，直接返回 Gotham 原生响应信封。
pub struct NativeResponseInterceptor;

impl Interceptor for NativeResponseInterceptor {
    fn intercept<'a>(
        &'a self,
        _invocation: Arc<Invocation>,
        _next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async {
            let mut response = Response::new("native-short-circuit".into_body());
            *response.status_mut() = StatusCode::ACCEPTED;
            Ok(Box::new(Arc::new(GothamResponse::new(response))) as InvocationValue)
        })
    }
}
