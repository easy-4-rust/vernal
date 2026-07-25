//! 应用上下文错误对象。

use std::{error::Error, fmt, time::Duration};

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
    /// 注册为应用事件监听器的组件不在 `IoC` 构建计划中。
    EventListenerDefinitionNotFound {
        /// 缺失的监听器组件标识。
        component: ComponentKey,
        /// 监听的事件 Rust 类型名。
        event: &'static str,
    },
    /// 应用事件监听器使用了无法表达 Context 级任务所有权的作用域。
    EventListenerScope {
        /// 作用域不合法的监听器组件身份。
        component: ComponentKey,
        /// 被监听事件的 Rust 类型名。
        event: &'static str,
        /// 稳定、无业务数据的作用域名称。
        scope: &'static str,
    },
    /// 同一组件与事件类型的监听关系被重复声明。
    DuplicateEventListener {
        /// 重复声明的监听器组件身份。
        component: ComponentKey,
        /// 被重复监听的事件 Rust 类型名。
        event: &'static str,
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
    /// 应用事件监听器组件无法从容器解析。
    EventListenerResolution {
        /// 正在解析的监听器组件标识。
        component: ComponentKey,
        /// 监听的事件 Rust 类型名。
        event: &'static str,
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
    /// 组件生命周期钩子超过执行预算并已请求 Tokio abort。
    LifecycleTimeout {
        /// 组件诊断名称。
        component: &'static str,
        /// 超时阶段。
        phase: LifecyclePhase,
        /// 该阶段配置的最长执行时间。
        timeout: Duration,
        /// Tokio 任务是否在 abort 收口预算内到达终态。
        abort_settled: bool,
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
    /// refresh 或 start 期间收到应用取消信号并完成回滚。
    LifecycleCancelled {
        /// 被取消的生命周期操作。
        operation: &'static str,
    },
    /// Tokio 无法注册或继续观察操作系统关闭信号。
    ShutdownSignal {
        /// 原始操作系统信号监听错误。
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
            Self::EventListenerDefinitionNotFound { component, event } => write!(
                formatter,
                "application event listener component is not registered in IoC: \
                 {component} for {event}"
            ),
            Self::EventListenerScope {
                component,
                event,
                scope,
            } => write!(
                formatter,
                "application event listener component {component} for {event} must be \
                singleton, not {scope}"
            ),
            Self::DuplicateEventListener { component, event } => write!(
                formatter,
                "application event listener {component} is registered more than once for {event}"
            ),
            Self::ContainerWarmUp { source } => {
                write!(formatter, "failed to warm up IoC container: {source}")
            }
            Self::ComponentResolution { component, source } => {
                write!(
                    formatter,
                    "failed to resolve lifecycle component {component}: {source}"
                )
            }
            Self::EventListenerResolution {
                component,
                event,
                source,
            } => write!(
                formatter,
                "failed to resolve application event listener {component} for {event}: {source}"
            ),
            Self::Lifecycle {
                component,
                phase,
                source,
            } => write!(
                formatter,
                "lifecycle component {component} failed during {phase}: {source}"
            ),
            Self::LifecycleTimeout {
                component,
                phase,
                timeout,
                abort_settled,
            } => write!(
                formatter,
                "lifecycle component {component} exceeded {phase} timeout {timeout:?}; \
                 Tokio abort settled: {abort_settled}"
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
            Self::LifecycleCancelled { operation } => {
                write!(
                    formatter,
                    "context lifecycle operation {operation} was cancelled and rolled back"
                )
            }
            Self::ShutdownSignal { source } => {
                write!(
                    formatter,
                    "failed to observe system shutdown signal: {source}"
                )
            }
        }
    }
}

impl Error for ContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ContainerWarmUp { source }
            | Self::ComponentResolution { source, .. }
            | Self::EventListenerResolution { source, .. } => Some(source.as_ref()),
            Self::Lifecycle { source, .. }
            | Self::LifecycleCoordinator { source, .. }
            | Self::ShutdownSignal { source } => Some(source.as_ref()),
            Self::ManagedTask { source } => Some(source),
            _ => None,
        }
    }
}
