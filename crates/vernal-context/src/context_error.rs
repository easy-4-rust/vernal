//! 应用上下文错误对象。

use std::{error::Error, fmt};

use vernal_core::SharedError;
use vernal_ioc::{ComponentKey, ResolveError};

use crate::{ContextState, LifecyclePhase, ManagedTaskError};

/// 应用上下文状态转换、组件解析或生命周期执行失败。
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ContextError {
    /// 当前状态不允许执行请求操作。
    InvalidState {
        /// 操作名称。
        operation: &'static str,
        /// 实际上下文状态。
        state: ContextState,
    },
    /// 注册为生命周期组件的类型不在 `IoC` 构建计划中。
    LifecycleDefinitionNotFound {
        /// 缺失的组件标识。
        component: ComponentKey,
    },
    /// `IoC` 单例预热失败。
    ContainerWarmUp {
        /// `IoC` 原始解析错误。
        source: Box<ResolveError>,
    },
    /// 生命周期组件无法从容器解析。
    ComponentResolution {
        /// 正在解析的组件标识。
        component: ComponentKey,
        /// `IoC` 原始解析错误。
        source: Box<ResolveError>,
    },
    /// 组件生命周期钩子执行失败。
    Lifecycle {
        /// 组件诊断名称。
        component: &'static str,
        /// 失败阶段。
        phase: LifecyclePhase,
        /// 原始组件错误。
        source: SharedError,
    },
    /// 应用受管 Tokio 任务执行或停机失败。
    ManagedTask {
        /// 任务监督器返回的结构化错误。
        source: ManagedTaskError,
    },
    /// 提交或观察 Tokio 生命周期协调任务失败。
    LifecycleCoordinator {
        /// 正在协调的 Context 操作。
        operation: &'static str,
        /// Runtime 缺失、协调任务 panic 或结果通道异常等原始错误。
        source: SharedError,
    },
}

impl fmt::Display for ContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState { operation, state } => {
                write!(
                    formatter,
                    "context operation {operation} is invalid in state {state:?}"
                )
            }
            Self::LifecycleDefinitionNotFound { component } => {
                write!(
                    formatter,
                    "lifecycle component is not registered in IoC: {component}"
                )
            }
            Self::ContainerWarmUp { source } => {
                write!(formatter, "failed to warm up IoC container: {source}")
            }
            Self::ComponentResolution { component, source } => {
                write!(
                    formatter,
                    "failed to resolve lifecycle component {component}: {source}"
                )
            }
            Self::Lifecycle {
                component,
                phase,
                source,
            } => write!(
                formatter,
                "lifecycle component {component} failed during {phase}: {source}"
            ),
            Self::ManagedTask { source } => {
                write!(formatter, "managed task lifecycle failed: {source}")
            }
            Self::LifecycleCoordinator { operation, source } => {
                write!(
                    formatter,
                    "context lifecycle coordinator failed during {operation}: {source}"
                )
            }
        }
    }
}

impl Error for ContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ContainerWarmUp { source } | Self::ComponentResolution { source, .. } => {
                Some(source.as_ref())
            }
            Self::Lifecycle { source, .. } | Self::LifecycleCoordinator { source, .. } => {
                Some(source.as_ref())
            }
            Self::ManagedTask { source } => Some(source),
            _ => None,
        }
    }
}
