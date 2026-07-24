//! 生命周期异步返回合同。

use std::{future::Future, pin::Pin};

use vernal_core::BoxError;

/// 可以借用组件自身的线程安全生命周期 Future。
pub type LifecycleFuture<'a> = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'a>>;
