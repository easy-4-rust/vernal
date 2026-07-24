//! 本地调用链错误对象。

use std::{error::Error, fmt};

use crate::Operation;

/// `!Send` AOP 调用链可观察的结构化失败。
///
/// 该错误与 [`crate::InvocationError`] 保持同一分类语义，但目标错误不要求
/// `Send + Sync`，从而可以原样承载 Actix Web 等 Worker-local 框架错误。
#[derive(Debug)]
#[non_exhaustive]
pub enum LocalInvocationError {
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
    /// 当前应用没有为目标操作预编译本地调用计划。
    PlanNotFound {
        /// 缺少计划的组件操作。
        operation: Operation,
    },
    /// 最终目标被重复推进。
    TargetAlreadyInvoked {
        /// 被重复执行的组件操作。
        operation: Operation,
    },
    /// 目标方法或本地拦截器返回业务错误。
    Target {
        /// 不要求跨线程移动的原始错误。
        source: Box<dyn Error>,
    },
    /// 擦除后的返回值无法恢复成调用方期待的类型。
    ReturnTypeMismatch {
        /// 期待的 Rust 类型名。
        expected: &'static str,
    },
}

impl LocalInvocationError {
    /// 将任意本地错误包装成目标执行错误。
    pub fn target(error: impl Error + 'static) -> Self {
        Self::Target {
            source: Box::new(error),
        }
    }

    /// 尝试从目标执行错误中恢复指定的原始错误类型。
    ///
    /// # Errors
    ///
    /// 当前错误不是 [`LocalInvocationError::Target`]，或内部类型不是 `T` 时，
    /// 返回未被消费的原始错误。
    pub fn into_target<T>(self) -> Result<T, Self>
    where
        T: Error + 'static,
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

impl fmt::Display for LocalInvocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("local invocation cancelled"),
            Self::DeadlineExceeded => formatter.write_str("local invocation deadline exceeded"),
            Self::PlanMismatch { expected, actual } => {
                write!(
                    formatter,
                    "local invocation plan mismatch: expected {expected}, got {actual}"
                )
            }
            Self::PlanNotFound { operation } => {
                write!(formatter, "local invocation plan not found: {operation}")
            }
            Self::TargetAlreadyInvoked { operation } => {
                write!(
                    formatter,
                    "local invocation target already executed: {operation}"
                )
            }
            Self::Target { source } => {
                write!(formatter, "local invocation target failed: {source}")
            }
            Self::ReturnTypeMismatch { expected } => {
                write!(
                    formatter,
                    "local invocation return type mismatch: expected {expected}"
                )
            }
        }
    }
}

impl Error for LocalInvocationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Target { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}
