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

    /// 尝试从目标执行错误中恢复指定的原始错误类型。
    ///
    /// Tower 等适配层可先用自己的私有包装类型标记下游错误，待整个拦截器链
    /// 结束后再恢复原生错误。这样既允许拦截器观察错误，又不会把适配层的
    /// `Service::Error` 永久擦除。类型不匹配或当前错误并非目标错误时，完整
    /// 返回原错误，调用方不会丢失诊断信息。
    ///
    /// # Errors
    ///
    /// 当前错误不是 [`InvocationError::Target`]，或内部错误类型不是 `T` 时，
    /// 返回未被消费的原始 [`InvocationError`]。
    pub fn into_target<T>(self) -> Result<T, Self>
    where
        T: Error + Send + Sync + 'static,
    {
        match self {
            Self::Target { source } => match source.downcast::<T>() {
                Ok(source) => Ok(*source),
                Err(source) => Err(Self::Target { source }),
            },
            other => Err(other),
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
