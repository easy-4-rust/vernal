//! 自定义作用域异步关闭合同。

use std::{future::Future, pin::Pin};

use vernal_core::BoxError;

/// 单个自定义作用域关闭钩子的异步返回值。
pub type ScopeFuture = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'static>>;

/// 只会执行一次的自定义作用域关闭钩子。
pub(crate) type ScopeCloseHook = Box<dyn FnOnce() -> ScopeFuture + Send + 'static>;
