//! 测试用本地调用顺序记录拦截器对象。

use std::sync::Arc;

use tokio::sync::Mutex;
use vernal_aop::{Invocation, LocalInterceptor, LocalInvocationFuture, LocalNext};

/// 在推进本地调用链前后记录稳定事件。
pub struct RecordingLocalInterceptor {
    name: &'static str,
    events: Arc<Mutex<Vec<String>>>,
}

impl RecordingLocalInterceptor {
    /// 创建共享事件记录拦截器。
    pub fn new(name: &'static str, events: Arc<Mutex<Vec<String>>>) -> Self {
        Self { name, events }
    }
}

impl LocalInterceptor for RecordingLocalInterceptor {
    fn intercept_local<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: LocalNext<'a>,
    ) -> LocalInvocationFuture<'a> {
        Box::pin(async move {
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
