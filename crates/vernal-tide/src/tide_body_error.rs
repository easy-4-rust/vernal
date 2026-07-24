//! Tide 响应 Body 清理错误对象。

use std::{error::Error, fmt};

use tokio::task::JoinError;
use vernal_web::ScopeError;

/// 区分请求 Scope 关闭失败与 Tokio 清理任务失败。
#[derive(Debug)]
pub enum TideBodyError {
    /// Body 结束后关闭请求 Scope 失败。
    Scope(ScopeError),
    /// 承载异步关闭操作的 Tokio 任务被取消或 panic。
    CleanupTask(JoinError),
}

impl fmt::Display for TideBodyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scope(error) => write!(formatter, "Tide request scope cleanup failed: {error}"),
            Self::CleanupTask(error) => {
                write!(formatter, "Tide request scope cleanup task failed: {error}")
            }
        }
    }
}

impl Error for TideBodyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Scope(error) => Some(error),
            Self::CleanupTask(error) => Some(error),
        }
    }
}
