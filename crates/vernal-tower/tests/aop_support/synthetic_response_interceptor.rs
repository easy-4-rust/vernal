//! 测试用响应短路拦截器对象。

use std::{cell::Cell, sync::Arc};

use http::Response;
use vernal_aop::{Interceptor, Invocation, InvocationFuture, InvocationValue, Next};
use vernal_tower::TowerResponse;

/// 不调用下游 Handler，直接产生一个原生 Tower 响应。
pub struct SyntheticResponseInterceptor {
    value: u8,
}

impl SyntheticResponseInterceptor {
    /// 创建携带指定响应值的短路拦截器。
    pub const fn new(value: u8) -> Self {
        Self { value }
    }
}

impl Interceptor for SyntheticResponseInterceptor {
    fn intercept<'a>(
        &'a self,
        _invocation: Arc<Invocation>,
        _next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            let response = Response::builder()
                .status(202)
                .header("x-vernal-short-circuit", "true")
                .body(Cell::new(self.value))
                .expect("synthetic response");
            Ok(Box::new(TowerResponse::new(response)) as InvocationValue)
        })
    }
}
