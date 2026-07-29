//! 异步运行时模块。
//!
//! 对标 Spring `org.springframework.core.task` 异步执行层。

mod runtime_type;
#[cfg(feature = "async-runtime")]
mod tokio_runtime;

pub use runtime_type::RuntimeType;
#[cfg(feature = "async-runtime")]
pub use tokio_runtime::TokioRuntime;
