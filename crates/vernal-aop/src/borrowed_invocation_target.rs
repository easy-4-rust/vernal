//! 借用当前异步调用期资源的线程安全目标契约。

use std::sync::Arc;

use crate::{Invocation, InvocationFuture};

/// 面向 Salvo `FlowCtrl` 等非静态、但可跨线程移动的最终目标。
///
/// 普通 [`crate::InvocationTarget`] 适合拥有全部数据的 `'static` 闭包。本契约让
/// Web Adapter 在一次调用中借用框架原生 Request、Response 或控制器，同时继续
/// 使用 Send-AOP。返回 Future 的生命周期绑定到 `&mut self`，因此借用不能逃出
/// [`crate::InvocationPlan::invoke_borrowed`] 的一次 `.await`。
///
/// 目标只要求 `Send`，不要求 `Sync`：`Next` 按值消费，完整 Around 链只会顺序
/// 推进这个独占目标一次。
pub trait BorrowedInvocationTarget: Send {
    /// 调用持有当前异步调用期借用的最终目标。
    fn invoke(&mut self, invocation: Arc<Invocation>) -> InvocationFuture<'_>;
}
