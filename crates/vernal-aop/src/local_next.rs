//! 本地拦截器链后继对象。

use std::sync::Arc;

use crate::{Invocation, LocalInterceptor, LocalInvocationFuture, LocalInvocationTarget};

/// 指向本地拦截器链中下一个节点。
///
/// `LocalNext` 按值消费，与线程安全 [`crate::Next`] 保持相同的一次推进和栈式
/// Around 语义，但不会要求下游 Future 或返回值实现 `Send`。
pub struct LocalNext<'a> {
    interceptors: &'a [Arc<dyn LocalInterceptor>],
    target: &'a LocalInvocationTarget,
    index: usize,
}

impl<'a> LocalNext<'a> {
    /// 从本地调用计划首节点创建后继对象。
    pub(crate) fn new(
        interceptors: &'a [Arc<dyn LocalInterceptor>],
        target: &'a LocalInvocationTarget,
    ) -> Self {
        Self {
            interceptors,
            target,
            index: 0,
        }
    }

    /// 推进到下一个本地拦截器；链尾调用最终目标。
    #[must_use]
    pub fn run(self, invocation: Arc<Invocation>) -> LocalInvocationFuture<'a> {
        Box::pin(async move {
            if let Some(interceptor) = self.interceptors.get(self.index) {
                let next = Self {
                    interceptors: self.interceptors,
                    target: self.target,
                    index: self.index + 1,
                };
                interceptor.intercept_local(invocation, next).await
            } else {
                (self.target)(invocation).await
            }
        })
    }
}
