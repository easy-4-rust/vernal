//! 本地调用链返回契约。

use std::{any::Any, future::Future, pin::Pin, sync::Arc};

use crate::{Invocation, LocalInvocationError};

/// 经过类型擦除、只在当前线程调用链中移动的业务返回值。
pub type LocalInvocationValue = Box<dyn Any>;

/// 本地拦截器链和目标方法共享的结果类型。
pub type LocalInvocationResult = Result<LocalInvocationValue, LocalInvocationError>;

/// 可以借用当前本地调用计划的 `!Send` 异步返回值。
pub type LocalInvocationFuture<'a> = Pin<Box<dyn Future<Output = LocalInvocationResult> + 'a>>;

/// 只在当前线程推进的最终目标函数。
///
/// 目标 Future 和返回值均不要求 `Send`，因此可以持有 `Rc`、Actix
/// `ServiceRequest` 或其他 Worker-local 原生对象；`Invocation` 仍使用 `Arc`
/// 共享 owned 快照、取消令牌与 deadline。
pub type LocalInvocationTarget =
    dyn Fn(Arc<Invocation>) -> LocalInvocationFuture<'static> + 'static;
