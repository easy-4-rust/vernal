//! tokio 运行时适配。
//!
//! 对标 Spring `org.springframework.core.task.support.AsyncTaskExecutor`。
//! 底层使用 `tokio` crate（对标 reactor-core）。

#![cfg(feature = "async-runtime")]

use std::future::Future;
use std::pin::Pin;

use super::runtime_type::RuntimeType;

/// tokio 运行时适配。
///
/// 对应 Java: `org.springframework.core.task.support.AsyncTaskExecutor`（简化版）
pub struct TokioRuntime;

impl TokioRuntime {
    /// 创建新的 tokio 运行时。
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 获取运行时类型。
    #[must_use]
    pub fn runtime_type(&self) -> RuntimeType {
        RuntimeType::Tokio
    }

    /// 异步执行任务。
    pub fn spawn_boxed(
        &self,
        future: Pin<Box<dyn Future<Output = ()> + Send>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(future)
    }

    /// 异步执行任务（使用闭包）。
    pub async fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(future)
    }

    /// 异步执行阻塞任务。
    pub async fn spawn_blocking<F, R>(&self, func: F) -> tokio::task::JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        tokio::task::spawn_blocking(func)
    }
}

impl Default for TokioRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_type_is_tokio() {
        let runtime = TokioRuntime::new();
        assert_eq!(runtime.runtime_type(), RuntimeType::Tokio);
    }

    #[test]
    fn default_impl_works() {
        let _runtime: TokioRuntime = TokioRuntime::default();
    }

    #[tokio::test]
    async fn spawn_returns_handle() {
        let runtime = TokioRuntime::new();
        let handle = runtime.spawn(async { 42 }).await;
        assert_eq!(handle.await.unwrap(), 42);
    }

    #[tokio::test]
    async fn spawn_blocking_returns_handle() {
        let runtime = TokioRuntime::new();
        let handle = runtime.spawn_blocking(|| 100).await;
        assert_eq!(handle.await.unwrap(), 100);
    }
}
