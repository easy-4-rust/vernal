//! 测试用调用顺序记录拦截器对象。

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::Mutex;
use vernal_aop::{Interceptor, Invocation, InvocationFuture, Next};
use vernal_web::RequestContext;

/// 记录 before/after，并验证 Web 请求上下文已进入 AOP 扩展。
pub struct RecordingInterceptor {
    name: &'static str,
    events: Arc<Mutex<Vec<String>>>,
    saw_request_context: Arc<AtomicBool>,
}

impl RecordingInterceptor {
    /// 创建测试拦截器。
    pub fn new(
        name: &'static str,
        events: Arc<Mutex<Vec<String>>>,
        saw_request_context: Arc<AtomicBool>,
    ) -> Self {
        Self {
            name,
            events,
            saw_request_context,
        }
    }
}

impl Interceptor for RecordingInterceptor {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            self.saw_request_context.store(
                invocation
                    .context()
                    .get::<Arc<RequestContext>>()
                    .await
                    .is_some(),
                Ordering::SeqCst,
            );
            self.events
                .lock()
                .await
                .push(format!("{}:before", self.name));
            let result = next.run(invocation).await;
            self.events
                .lock()
                .await
                .push(format!("{}:after", self.name));
            result
        })
    }
}
