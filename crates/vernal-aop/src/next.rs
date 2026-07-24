//! 拦截器链后继对象。

use std::sync::Arc;

use crate::{Interceptor, Invocation, InvocationFuture, target_ref::TargetRef};

/// 指向拦截器链中下一个节点。
///
/// `Next` 按值消费，确保同一个拦截器不会意外重复推进同一条链。顺序较小的
/// 拦截器先进入、后退出，自然形成与 Spring `Around` 一致的栈式语义。
pub struct Next<'a> {
    interceptors: &'a [Arc<dyn Interceptor>],
    target: TargetRef<'a>,
    index: usize,
}

impl<'a> Next<'a> {
    /// 从调用计划首节点创建后继对象。
    pub(crate) fn new(interceptors: &'a [Arc<dyn Interceptor>], target: TargetRef<'a>) -> Self {
        Self {
            interceptors,
            target,
            index: 0,
        }
    }

    /// 推进到下一个拦截器；链尾调用最终目标。
    #[must_use]
    pub fn run(self, invocation: Arc<Invocation>) -> InvocationFuture<'a> {
        Box::pin(async move {
            if let Some(interceptor) = self.interceptors.get(self.index) {
                let next = Self {
                    interceptors: self.interceptors,
                    target: self.target,
                    index: self.index + 1,
                };
                interceptor.intercept(invocation, next).await
            } else {
                self.target.invoke(invocation).await
            }
        })
    }
}
