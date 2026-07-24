//! Web 请求作用域错误对象。

use std::{error::Error, fmt};

use vernal_core::BoxError;

use crate::ScopeState;

/// 请求作用域解析、注册或关闭失败。
#[derive(Debug)]
#[non_exhaustive]
pub enum ScopeError {
    /// 当前状态不允许执行操作。
    InvalidState {
        /// 操作名称。
        operation: &'static str,
        /// 实际状态。
        state: ScopeState,
    },
    /// 同一 `TypeId` 下缓存了无法恢复成期望类型的对象。
    TypeMismatch {
        /// 期望的 Rust 类型名。
        expected: &'static str,
    },
    /// 一个异步关闭钩子失败。
    CloseHook {
        /// 原始关闭错误。
        source: BoxError,
    },
}

impl fmt::Display for ScopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState { operation, state } => {
                write!(
                    formatter,
                    "request scope operation {operation} is invalid in state {state:?}"
                )
            }
            Self::TypeMismatch { expected } => {
                write!(
                    formatter,
                    "request scope type mismatch: expected {expected}"
                )
            }
            Self::CloseHook { source } => write!(formatter, "request scope close failed: {source}"),
        }
    }
}

impl Error for ScopeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CloseHook { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}
