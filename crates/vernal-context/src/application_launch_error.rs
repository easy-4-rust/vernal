//! Vernal 应用统一启动错误对象。

use std::{error::Error, fmt};

use vernal_core::SharedError;

use crate::{ApplicationBuildError, ContextError, StartupReport};

/// 高层应用在构建、refresh、start 或启动协调阶段的结构化失败。
///
/// 构建失败发生在 `ApplicationContext` 创建之前，因此没有启动报告。其余失败都
/// 保存失败后的脱敏报告和可选清理错误：主错误始终表示最先失败的启动阶段，清理
/// 错误只作为补充证据，不能覆盖原始根因。
#[derive(Debug)]
#[non_exhaustive]
pub enum ApplicationLaunchError {
    /// 依赖图、条件模块、AOP 计划或 Context 声明无法完成构建。
    Build {
        /// 高层应用建造器返回的原始错误。
        source: ApplicationBuildError,
    },
    /// Context 在 refresh 或 start 阶段失败。
    Lifecycle {
        /// 稳定、低基数的启动阶段名称。
        operation: &'static str,
        /// refresh 或 start 返回的原始错误。
        source: ContextError,
        /// 失败后执行幂等关闭时观察到的补充错误。
        cleanup: Option<ContextError>,
        /// 清理完成后取得的脱敏启动报告。
        report: StartupReport,
    },
    /// 持有完整启动流程的 Tokio 协调任务异常结束。
    Coordinator {
        /// Tokio `JoinError` 或结果通道异常等原始协调错误。
        source: SharedError,
        /// 协调异常后执行幂等关闭时观察到的补充错误。
        cleanup: Option<ContextError>,
        /// 清理完成后取得的脱敏启动报告。
        report: StartupReport,
    },
}

impl ApplicationLaunchError {
    /// 创建构建阶段错误。
    pub(crate) const fn build(source: ApplicationBuildError) -> Self {
        Self::Build { source }
    }

    /// 创建 refresh 或 start 阶段错误。
    pub(crate) const fn lifecycle(
        operation: &'static str,
        source: ContextError,
        cleanup: Option<ContextError>,
        report: StartupReport,
    ) -> Self {
        Self::Lifecycle {
            operation,
            source,
            cleanup,
            report,
        }
    }

    /// 创建启动协调任务错误。
    pub(crate) fn coordinator(
        source: impl Error + Send + Sync + 'static,
        cleanup: Option<ContextError>,
        report: StartupReport,
    ) -> Self {
        Self::Coordinator {
            source: std::sync::Arc::new(source),
            cleanup,
            report,
        }
    }

    /// 返回失败所在的稳定启动阶段。
    #[must_use]
    pub const fn operation(&self) -> &'static str {
        match self {
            Self::Build { .. } => "build",
            Self::Lifecycle { operation, .. } => operation,
            Self::Coordinator { .. } => "launch-coordinator",
        }
    }

    /// 返回 Context 已创建后产生的脱敏启动报告。
    ///
    /// 构建失败尚未形成 Context，因此返回 `None`。
    #[must_use]
    pub const fn startup_report(&self) -> Option<&StartupReport> {
        match self {
            Self::Build { .. } => None,
            Self::Lifecycle { report, .. } | Self::Coordinator { report, .. } => Some(report),
        }
    }

    /// 返回启动失败后幂等关闭观察到的补充错误。
    ///
    /// 该错误不会替代 [`Error::source`] 暴露的首个启动错误。
    #[must_use]
    pub const fn cleanup_error(&self) -> Option<&ContextError> {
        match self {
            Self::Build { .. } => None,
            Self::Lifecycle { cleanup, .. } | Self::Coordinator { cleanup, .. } => cleanup.as_ref(),
        }
    }
}

impl fmt::Display for ApplicationLaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build { source } => write!(formatter, "application build failed: {source}"),
            Self::Lifecycle {
                operation,
                source,
                cleanup,
                ..
            } => {
                write!(
                    formatter,
                    "application launch failed during {operation}: {source}"
                )?;
                if cleanup.is_some() {
                    formatter.write_str("; context cleanup also failed")?;
                }
                Ok(())
            }
            Self::Coordinator {
                source, cleanup, ..
            } => {
                write!(formatter, "application launch coordinator failed: {source}")?;
                if cleanup.is_some() {
                    formatter.write_str("; context cleanup also failed")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for ApplicationLaunchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Build { source } => Some(source),
            Self::Lifecycle { source, .. } => Some(source),
            Self::Coordinator { source, .. } => Some(source.as_ref()),
        }
    }
}
