//! 调用链返回契约。

use std::{any::Any, future::Future, pin::Pin, sync::Arc};

use crate::{Invocation, InvocationError};

/// 经过类型擦除、仍保持线程安全的业务返回值。
pub type InvocationValue = Box<dyn Any + Send + Sync>;

/// 拦截器链和目标方法共享的结果类型。
pub type InvocationResult = Result<InvocationValue, InvocationError>;

/// 可以借用当前拦截计划的异步返回值。
pub type InvocationFuture<'a> = Pin<Box<dyn Future<Output = InvocationResult> + Send + 'a>>;

/// 被拦截的最终目标函数。
///
/// 目标接收 owned `Arc<Invocation>`，返回 `'static` Future，避免把框架请求借用
/// 跨越 `.await`。需要请求数据时应从调用上下文读取 owned snapshot。
pub type InvocationTarget =
    dyn Fn(Arc<Invocation>) -> InvocationFuture<'static> + Send + Sync + 'static;
