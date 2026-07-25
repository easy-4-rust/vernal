//! 借用型异步业务 Future 目标对象。

use std::sync::Arc;

use crate::{BorrowedInvocationTarget, Invocation, InvocationError, InvocationFuture, Operation};

/// 把一个已经拥有参数、但仍借用方法接收器的 Future 适配为 AOP 最终目标。
///
/// `#[vernal_macros::intercept]` 可以在 `async fn(&self, ..)` 入口中先构造业务
/// Future，再把它交给 [`crate::InvocationPlan::invoke_borrowed`]。Future 的生命
/// 周期被绑定到当前方法调用，既不要求克隆接收器，也不会被提升为 `'static`。
///
/// 最终目标按合同最多执行一次。拦截器短路时 Future 会随本对象一起释放；错误地
/// 重复推进目标时返回结构化错误，不使用 panic 参与正常控制流。
pub struct BorrowedInvocationFutureTarget<'a> {
    operation: Operation,
    future: Option<InvocationFuture<'a>>,
}

impl<'a> BorrowedInvocationFutureTarget<'a> {
    /// 使用操作身份和待执行业务 Future 创建借用型目标。
    #[must_use]
    pub fn new(operation: Operation, future: InvocationFuture<'a>) -> Self {
        Self {
            operation,
            future: Some(future),
        }
    }
}

impl BorrowedInvocationTarget for BorrowedInvocationFutureTarget<'_> {
    /// 取得并推进唯一业务 Future；重复调用返回 `TargetAlreadyInvoked`。
    fn invoke(&mut self, _invocation: Arc<Invocation>) -> InvocationFuture<'_> {
        if let Some(future) = self.future.take() {
            return future;
        }
        let operation = self.operation.clone();
        Box::pin(async move { Err(InvocationError::TargetAlreadyInvoked { operation }) })
    }
}
