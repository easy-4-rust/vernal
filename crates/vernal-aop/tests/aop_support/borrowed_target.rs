//! 测试用非静态 Send-AOP 目标对象。

use std::sync::Arc;

use vernal_aop::{BorrowedInvocationTarget, Invocation, InvocationFuture, InvocationValue};

/// 独占借用测试栈上 Vec，证明 Send 目标不需要伪造成 `'static`。
pub struct BorrowedTarget<'a> {
    events: &'a mut Vec<&'static str>,
}

impl<'a> BorrowedTarget<'a> {
    /// 创建借用当前调用栈数据的目标。
    pub const fn new(events: &'a mut Vec<&'static str>) -> Self {
        Self { events }
    }
}

impl BorrowedInvocationTarget for BorrowedTarget<'_> {
    fn invoke(&mut self, _invocation: Arc<Invocation>) -> InvocationFuture<'_> {
        Box::pin(async move {
            self.events.push("borrowed-target:before");
            tokio::task::yield_now().await;
            self.events.push("borrowed-target:after");
            Ok(Box::new(String::from("borrowed-result")) as InvocationValue)
        })
    }
}
