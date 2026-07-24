//! 借用当前 Worker 调用期资源的本地目标契约。

use std::sync::Arc;

use crate::{Invocation, LocalInvocationFuture};

/// 面向 Ntex `ServiceCtx` 等非静态框架对象的本地最终目标。
///
/// 返回 Future 的生命周期绑定到 `&self`，因此实现对象可以安全持有当前
/// Pipeline 的借用，但借用绝不会逃出 [`crate::LocalInvocationPlan`] 的一次
/// `.await`。该 trait 不要求 `Send`、`Sync` 或 `'static`。
pub trait BorrowedLocalInvocationTarget {
    /// 调用持有当前 Worker 借用的最终目标。
    fn invoke(&self, invocation: Arc<Invocation>) -> LocalInvocationFuture<'_>;
}
