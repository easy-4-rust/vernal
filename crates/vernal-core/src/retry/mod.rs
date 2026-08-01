//! 重试支持包。
//!
//! 对标 Spring `org.springframework.core.retry` 包：重试策略、模板与监听器。

mod default_retry_policy;
mod retry_exception;
mod retry_listener;
mod retry_operations;
mod retry_policy;
mod retry_state;
mod retry_template;
pub mod support;

pub use default_retry_policy::DefaultRetryPolicy;
pub use retry_exception::RetryException;
pub use retry_listener::RetryListener;
pub use retry_operations::RetryOperations;
pub use retry_policy::{FixedBackOff, RetryPolicy};
pub use retry_state::{RetryState, SimpleRetryState};
pub use retry_template::RetryTemplate;
