//! Local-AOP 内部目标引用对象。

use std::sync::Arc;

use crate::{
    BorrowedLocalInvocationTarget, Invocation, LocalInvocationFuture, LocalInvocationTarget,
};

/// 在同一条拦截链内统一静态闭包目标与借用型目标。
#[derive(Clone, Copy)]
pub(crate) enum LocalTargetRef<'a> {
    /// 原有 `'static` 闭包目标。
    Static(&'a LocalInvocationTarget),
    /// 生命周期绑定到当前 Worker 调用的目标对象。
    Borrowed(&'a dyn BorrowedLocalInvocationTarget),
}

impl<'a> LocalTargetRef<'a> {
    /// 调用实际目标，并把静态 Future 安全收窄到当前链生命周期。
    pub(crate) fn invoke(self, invocation: Arc<Invocation>) -> LocalInvocationFuture<'a> {
        match self {
            Self::Static(target) => target(invocation),
            Self::Borrowed(target) => target.invoke(invocation),
        }
    }
}
