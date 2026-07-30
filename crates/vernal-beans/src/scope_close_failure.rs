//! 作用域关闭失败类型。

use std::sync::Arc;

/// 作用域关闭过程中发生的失败。
#[derive(Debug, Clone)]
pub enum ScopeCloseFailure {
    /// 关闭钩子执行失败。
    Hook(Arc<dyn std::error::Error + Send + Sync>),
    /// 异步任务执行失败。
    Task(Arc<dyn std::error::Error + Send + Sync>),
}
