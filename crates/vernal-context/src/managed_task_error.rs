//! 受管 Tokio 任务错误对象。

use std::{error::Error, fmt, time::Duration};

use vernal_core::SharedError;

/// 描述受管任务注册、执行或应用停机阶段的结构化失败。
///
/// 任务名称必须是静态字符串，Context 诊断只记录固定告警代码；原始错误仅通过
/// 调用方持有的错误链返回，不会自动进入可序列化启动报告。
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ManagedTaskError {
    /// 任务名为空，无法形成稳定诊断身份。
    InvalidName,
    /// Context-local 任务标识空间已经耗尽。
    IdentifierExhausted {
        /// 无法分配标识的静态任务名。
        task: &'static str,
    },
    /// 任务监督器已经开始排空，拒绝新的后台任务。
    SpawnRejected {
        /// 调用方声明的低基数静态任务名。
        task: &'static str,
    },
    /// 任务 Future 返回业务或基础设施错误。
    TaskFailed {
        /// 调用方声明的低基数静态任务名。
        task: &'static str,
        /// 原始任务错误。
        source: SharedError,
    },
    /// 任务 Future 发生 panic。
    TaskPanicked {
        /// 调用方声明的低基数静态任务名。
        task: &'static str,
        /// Tokio Join 错误。
        source: SharedError,
    },
    /// 任务在应用尚未取消时被外部终止。
    TaskCancelled {
        /// 调用方声明的低基数静态任务名。
        task: &'static str,
        /// Tokio Join 错误。
        source: SharedError,
    },
    /// 优雅等待到期后已向剩余任务发送强制终止。
    ShutdownTimeout {
        /// 优雅阶段等待上限。
        timeout: Duration,
        /// 进入强制终止阶段时仍存活的任务数。
        remaining: usize,
    },
    /// 发送强制终止后，任务观察器仍未在第二阶段上限内收口。
    AbortTimeout {
        /// 强制终止阶段等待上限。
        timeout: Duration,
        /// 第二阶段到期时仍未收口的任务数。
        remaining: usize,
    },
    /// 停机结果通道在发布终态前异常关闭。
    CoordinatorUnavailable {
        /// 通道关闭时仍未收口的任务数。
        remaining: usize,
    },
}

impl fmt::Display for ManagedTaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName => formatter.write_str("managed task name must not be empty"),
            Self::IdentifierExhausted { task } => {
                write!(
                    formatter,
                    "managed task identifier space exhausted for {task}"
                )
            }
            Self::SpawnRejected { task } => {
                write!(
                    formatter,
                    "managed task {task} was rejected during shutdown"
                )
            }
            Self::TaskFailed { task, source } => {
                write!(formatter, "managed task {task} failed: {source}")
            }
            Self::TaskPanicked { task, source } => {
                write!(formatter, "managed task {task} panicked: {source}")
            }
            Self::TaskCancelled { task, source } => {
                write!(
                    formatter,
                    "managed task {task} was cancelled unexpectedly: {source}"
                )
            }
            Self::ShutdownTimeout { timeout, remaining } => write!(
                formatter,
                "managed task shutdown exceeded {timeout:?}; aborted {remaining} remaining tasks"
            ),
            Self::AbortTimeout { timeout, remaining } => write!(
                formatter,
                "managed task abort did not settle within {timeout:?}; {remaining} tasks remain"
            ),
            Self::CoordinatorUnavailable { remaining } => write!(
                formatter,
                "managed task shutdown coordinator stopped; {remaining} tasks remain"
            ),
        }
    }
}

impl Error for ManagedTaskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TaskFailed { source, .. }
            | Self::TaskPanicked { source, .. }
            | Self::TaskCancelled { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
