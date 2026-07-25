//! 自定义组件作用域错误对象。

use std::{error::Error, fmt, time::Duration};

use vernal_core::SharedError;

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
        source: SharedError,
    },
    /// 承载关闭钩子的 Tokio 任务异常结束。
    CloseTask {
        /// 目标作用域身份。
        scope: ScopeKey,
        /// Tokio Join 错误。
        source: SharedError,
    },
    /// 调用方等待后台关闭完成超过明确上限。
    CloseTimeout {
        /// 目标作用域身份。
        scope: ScopeKey,
        /// 本次调用允许等待的最长时间。
        timeout: Duration,
    },
    /// 启动取消安全关闭任务时没有可用的 Tokio Runtime。
    RuntimeUnavailable {
        /// 目标作用域身份。
        scope: ScopeKey,
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
            Self::CloseTask { scope, source } => {
                write!(formatter, "scope {scope} close task failed: {source}")
            }
            Self::CloseTimeout { scope, timeout } => {
                write!(
                    formatter,
                    "scope {scope} cleanup exceeded timeout {timeout:?}"
                )
            }
            Self::RuntimeUnavailable { scope } => {
                write!(
                    formatter,
                    "scope {scope} cleanup requires an active Tokio runtime"
                )
            }
        }
    }
}

impl Error for ScopeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolution { source, .. } => Some(source.as_ref()),
            Self::CloseHook { source, .. } | Self::CloseTask { source, .. } => {
                Some(source.as_ref())
            }
            _ => None,
        }
    }
}
