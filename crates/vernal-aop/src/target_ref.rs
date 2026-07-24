//! Send-AOP 内部目标引用对象。

use std::sync::Arc;

use crate::{BorrowedInvocationTarget, Invocation, InvocationFuture, InvocationTarget};

/// 在同一条 Send 拦截链中统一静态闭包目标与独占借用型目标。
pub(crate) enum TargetRef<'a> {
    /// 原有 `Send + Sync + 'static` 闭包目标。
    Static(&'a InvocationTarget),
    /// 生命周期绑定到当前异步调用的独占借用目标。
    Borrowed(&'a mut dyn BorrowedInvocationTarget),
}

impl<'a> TargetRef<'a> {
    /// 调用实际目标，并把返回 Future 限制在当前链生命周期内。
    pub(crate) fn invoke(self, invocation: Arc<Invocation>) -> InvocationFuture<'a> {
        match self {
            Self::Static(target) => target(invocation),
            Self::Borrowed(target) => target.invoke(invocation),
        }
    }
}
