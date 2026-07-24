//! 自定义组件作用域错误对象。

use std::{error::Error, fmt};

use vernal_core::BoxError;

use crate::{ResolveError, ScopeKey, ScopeState};

/// 自定义作用域进入、缓存或关闭失败。
#[derive(Debug)]
#[non_exhaustive]
pub enum ScopeError {
    /// 当前状态不允许执行指定操作。
    InvalidState {
        /// 失败的作用域操作。
        operation: &'static str,
        /// 目标作用域身份。
        scope: ScopeKey,
        /// 实际状态。
        state: ScopeState,
    },
    /// 父作用域或当前作用域已经发出取消信号。
    Cancelled {
        /// 失败的作用域操作。
        operation: &'static str,
        /// 目标作用域身份。
        scope: ScopeKey,
    },
    /// 同一组件键下缓存了无法恢复成期望类型的对象。
    TypeMismatch {
        /// 目标作用域身份。
        scope: ScopeKey,
        /// 期望的 Rust 类型名。
        expected: &'static str,
    },
    /// 通用类型缓存命中了由 `IoC` 工厂记录的组件解析错误。
    Resolution {
        /// 目标作用域身份。
        scope: ScopeKey,
        /// 原始组件解析错误。
        source: Box<ResolveError>,
    },
    /// 一个异步关闭钩子失败。
    CloseHook {
        /// 目标作用域身份。
        scope: ScopeKey,
        /// 原始关闭错误。
        source: BoxError,
    },
}

impl fmt::Display for ScopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState {
                operation,
                scope,
                state,
            } => write!(
                formatter,
                "scope {scope} operation {operation} is invalid in state {state:?}"
            ),
            Self::Cancelled { operation, scope } => {
                write!(
                    formatter,
                    "scope {scope} operation {operation} was cancelled"
                )
            }
            Self::TypeMismatch { scope, expected } => {
                write!(
                    formatter,
                    "scope {scope} cached component type mismatch: expected {expected}"
                )
            }
            Self::Resolution { scope, source } => {
                write!(
                    formatter,
                    "scope {scope} component resolution failed: {source}"
                )
            }
            Self::CloseHook { scope, source } => {
                write!(formatter, "scope {scope} close hook failed: {source}")
            }
        }
    }
}

impl Error for ScopeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolution { source, .. } => Some(source.as_ref()),
            Self::CloseHook { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
