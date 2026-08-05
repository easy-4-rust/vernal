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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_display() {
        let err = InvocationError::Cancelled;
        assert_eq!(format!("{}", err), "invocation cancelled");
    }

    #[test]
    fn deadline_exceeded_display() {
        let err = InvocationError::DeadlineExceeded;
        assert_eq!(format!("{}", err), "invocation deadline exceeded");
    }

    #[test]
    fn plan_mismatch_display() {
        let err = InvocationError::PlanMismatch {
            expected: Operation::new("Service", "method1"),
            actual: Operation::new("Service", "method2"),
        };
        assert!(format!("{}", err).contains("plan mismatch"));
    }

    #[test]
    fn plan_not_found_display() {
        let err = InvocationError::PlanNotFound {
            operation: Operation::new("Service", "method"),
        };
        assert!(format!("{}", err).contains("plan not found"));
    }

    #[test]
    fn target_already_invoked_display() {
        let err = InvocationError::TargetAlreadyInvoked {
            operation: Operation::new("Service", "method"),
        };
        assert!(format!("{}", err).contains("already executed"));
    }

    #[test]
    fn target_display() {
        let err = InvocationError::target(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert!(format!("{}", err).contains("target failed"));
    }

    #[test]
    fn return_type_mismatch_display() {
        let err = InvocationError::ReturnTypeMismatch { expected: "i32" };
        assert!(format!("{}", err).contains("return type mismatch"));
    }

    #[test]
    fn target_source() {
        let err = InvocationError::target(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert!(err.source().is_some());
    }

    #[test]
    fn cancelled_source() {
        let err = InvocationError::Cancelled;
        assert!(err.source().is_none());
    }

    #[test]
    fn into_target_success() {
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let err = InvocationError::target(inner);
        let result: Result<std::io::Error, _> = err.into_target();
        assert!(result.is_ok());
    }

    #[test]
    fn into_target_wrong_type() {
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let err = InvocationError::target(inner);
        let result: Result<InvocationError, _> = err.into_target();
        assert!(result.is_err());
    }

    #[test]
    fn into_target_not_target_variant() {
        let err = InvocationError::Cancelled;
        let result: Result<std::io::Error, _> = err.into_target();
        assert!(result.is_err());
    }
}
