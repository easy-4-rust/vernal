//! 测试用非静态 Local-AOP 目标对象。

use std::{cell::RefCell, rc::Rc, sync::Arc};

use vernal_aop::{
    BorrowedLocalInvocationTarget, Invocation, LocalInvocationFuture, LocalInvocationValue,
};

/// 借用测试栈上 RefCell，证明目标不需要伪造成 `'static`。
pub struct BorrowedLocalTarget<'a> {
    events: &'a RefCell<Vec<&'static str>>,
}

impl<'a> BorrowedLocalTarget<'a> {
    /// 创建借用当前调用栈数据的目标。
    pub const fn new(events: &'a RefCell<Vec<&'static str>>) -> Self {
        Self { events }
    }
}

impl BorrowedLocalInvocationTarget for BorrowedLocalTarget<'_> {
    fn invoke(&self, _invocation: Arc<Invocation>) -> LocalInvocationFuture<'_> {
        Box::pin(async move {
            self.events.borrow_mut().push("borrowed-target");
            tokio::task::yield_now().await;
            Ok(Box::new(Rc::new(String::from("borrowed-result"))) as LocalInvocationValue)
        })
    }
}
