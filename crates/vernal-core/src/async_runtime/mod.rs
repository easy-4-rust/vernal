//! 异步运行时抽象模块（feature = "async-runtime"）。
//!
//! 对标 Spring `reactor-core` / `rxjava` / `mutiny`。
//!
//! # 与 Spring 的对应关系
//!
//! | Spring | vernal-core |
//! |---|---|
//! | `reactor.core.publisher.Mono<T>` | `async fn -> Result<T, E>` |
//! | `reactor.core.publisher.Flux<T>` | `tokio_stream::Stream<Item = Result<T, E>>` |
//! | `reactor.core.scheduler.Schedulers` | `tokio::runtime::Runtime` |
//! | `io.reactivex.rxjava3.core.Observable` | `tokio_stream::Stream` |
//! | `io.smallrye.reactive.mutiny.Uni<T>` | `async fn -> Result<T, E>` |

/// 异步运行时类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeType {
    /// Tokio 运行时（默认）
    Tokio,
    /// async-std 运行时
    AsyncStd,
    /// 当前线程运行时
    CurrentThread,
}

impl RuntimeType {
    /// 获取运行时类型的字符串表示。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tokio => "tokio",
            Self::AsyncStd => "async-std",
            Self::CurrentThread => "current-thread",
        }
    }
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Tokio 异步运行时封装（feature = "async-runtime"）。
///
/// 对标 Spring `reactor.core.scheduler.Schedulers`。
///
/// 使用 `tokio` crate 作为后端，提供异步任务执行功能。
#[cfg(feature = "async-runtime")]
pub mod tokio_runtime {
    use super::RuntimeType;

    /// Tokio 异步运行时封装。
    ///
    /// 对标 Spring `Schedulers`。
    #[derive(Debug)]
    pub struct TokioRuntime {
        runtime_type: RuntimeType,
    }

    impl TokioRuntime {
        /// 创建新的 Tokio 运行时。
        pub fn new() -> Self {
            Self {
                runtime_type: RuntimeType::Tokio,
            }
        }

        /// 获取运行时类型。
        pub fn runtime_type(&self) -> RuntimeType {
            self.runtime_type
        }

        /// 异步执行任务。
        ///
        /// 对标 Spring `Schedulers.parallel().schedule(task)`。
        pub async fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
        where
            F: std::future::Future + Send + 'static,
            F::Output: Send + 'static,
        {
            tokio::spawn(future)
        }

        /// 在当前线程执行任务。
        ///
        /// 对标 Spring `Schedulers.immediate().schedule(task)`。
        pub fn spawn_blocking<F, R>(&self, f: F) -> tokio::task::JoinHandle<R>
        where
            F: FnOnce() -> R + Send + 'static,
            R: Send + 'static,
        {
            tokio::task::spawn_blocking(f)
        }
    }

    impl Default for TokioRuntime {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_type_display() {
        assert_eq!(RuntimeType::Tokio.to_string(), "tokio");
        assert_eq!(RuntimeType::AsyncStd.to_string(), "async-std");
        assert_eq!(RuntimeType::CurrentThread.to_string(), "current-thread");
    }

    #[test]
    fn runtime_type_equality() {
        assert_eq!(RuntimeType::Tokio, RuntimeType::Tokio);
        assert_ne!(RuntimeType::Tokio, RuntimeType::AsyncStd);
    }
}
