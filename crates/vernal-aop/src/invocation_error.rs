//! 调用链错误对象。

use std::{error::Error, fmt};

use vernal_core::BoxError;

use crate::Operation;

/// AOP 调用链可观察的结构化失败。
#[derive(Debug)]
#[non_exhaustive]
pub enum InvocationError {
    /// 调用在完成前被取消。
    Cancelled,
    /// 调用超过绝对截止时间。
    DeadlineExceeded,
    /// 调用计划与实际操作不匹配。
    PlanMismatch {
        /// 计划绑定的操作。
        expected: Operation,
        /// 实际调用操作。
        actual: Operation,
    },
    /// 当前应用没有为目标操作预编译调用计划。
    PlanNotFound {
        /// 缺少计划的组件操作。
        operation: Operation,
    },
    /// 最终目标被重复推进，已拥有的参数不能再次消费。
    TargetAlreadyInvoked {
        /// 被重复执行的组件操作。
        operation: Operation,
    },
    /// 目标方法或拦截器返回业务错误。
    Target {
        /// 原始错误。
        source: BoxError,
    },
    /// 擦除后的返回值无法恢复成调用方期待的类型。
    ReturnTypeMismatch {
        /// 期待的 Rust 类型名。
        expected: &'static str,
    },
}

impl InvocationError {
    /// 将任意线程安全错误包装成目标执行错误。
    pub fn target(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Target {
            source: Box::new(error),
        }
    }
}

impl fmt::Display for InvocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("invocation cancelled"),
            Self::DeadlineExceeded => formatter.write_str("invocation deadline exceeded"),
            Self::PlanMismatch { expected, actual } => {
                write!(
                    formatter,
                    "invocation plan mismatch: expected {expected}, got {actual}"
                )
            }
            Self::PlanNotFound { operation } => {
                write!(formatter, "invocation plan not found: {operation}")
            }
            Self::TargetAlreadyInvoked { operation } => {
                write!(formatter, "invocation target already executed: {operation}")
            }
            Self::Target { source } => write!(formatter, "invocation target failed: {source}"),
            Self::ReturnTypeMismatch { expected } => {
                write!(
                    formatter,
                    "invocation return type mismatch: expected {expected}"
                )
            }
        }
    }
}

impl Error for InvocationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Target { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}
