//! 应用上下文错误对象。

use std::{error::Error, fmt, time::Duration};

use vernal_beans::{ComponentKey, ResolveError};
use vernal_core::SharedError;

use crate::{ApplicationRunnerFailure, ContextState, LifecyclePhase, ManagedTaskError};

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
    /// 注册为应用 Runner 的组件不在 `IoC` 构建计划中。
    ApplicationRunnerDefinitionNotFound {
        /// 缺失的 Runner 组件标识。
        component: ComponentKey,
    },
    /// 应用 Runner 使用了无法表达应用级单例身份的作用域。
    ApplicationRunnerScope {
        /// 作用域不合法的 Runner 组件身份。
        component: ComponentKey,
        /// 稳定、无业务数据的作用域名称。
        scope: &'static str,
    },
    /// 同一 Runner 组件被重复声明。
    DuplicateApplicationRunner {
        /// 重复声明的 Runner 组件身份。
        component: ComponentKey,
    },
    /// 注册为周期任务的组件不在 `IoC` 构建计划中。
    ScheduledTaskDefinitionNotFound {
        /// 缺失的周期任务组件标识。
        component: ComponentKey,
    },
    /// 周期任务使用了无法表达应用级后台任务身份的作用域。
    ScheduledTaskScope {
        /// 作用域不合法的任务组件身份。
        component: ComponentKey,
        /// 稳定、无业务数据的作用域名称。
        scope: &'static str,
    },
    /// 同一周期任务组件被重复声明。
    DuplicateScheduledTask {
        /// 重复声明的任务组件身份。
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
    /// 应用 Runner 组件无法从最终 Container 解析。
    ApplicationRunnerResolution {
        /// 正在解析的 Runner 组件标识。
        component: ComponentKey,
        /// `IoC` 原始解析错误。
        source: Box<ResolveError>,
    },
    /// 周期任务组件无法从最终 Container 解析。
    ScheduledTaskResolution {
        /// 正在解析的任务组件标识。
        component: ComponentKey,
        /// `IoC` 原始解析错误。
        source: Box<ResolveError>,
    },
    /// 应用 Runner 返回错误或其 Tokio task 发生 panic/异常取消。
    ApplicationRunnerFailed {
        /// 默认格式化脱敏、显式错误链保留根因的失败对象。
        source: ApplicationRunnerFailure,
    },
    /// 应用 Runner 超过启动阶段预算并已请求 Tokio abort。
    ApplicationRunnerTimeout {
        /// Runner 提供的低基数静态诊断名称。
        runner: &'static str,
        /// 复用的生命周期启动阶段最长执行时间。
        timeout: Duration,
        /// Tokio task 是否在 abort 收口预算内到达终态。
        abort_settled: bool,
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
    /// `pause()` 或 `restart()` 期间组件钩子执行失败并完成回滚。
    ///
    /// 对标 Spring `LifecycleProcessor.onPause()` / `onRestart()` 期间抛出的
    /// `ApplicationContextException`。错误携带触发的操作名（`pause` 或
    /// `restart`）、首个失败组件、阶段和原始错误源；后续失败保留在诊断
    /// `StartupReport::observations` 中，不进入公开 Display 输出。
    PauseRestart {
        /// 触发失败的操作：`pause` 或 `restart`。
        operation: &'static str,
        /// 首个失败组件诊断名。
        component: &'static str,
        /// 失败阶段：`Pause` / `Start` / `Stop`。
        phase: LifecyclePhase,
        /// 默认格式化脱敏、显式错误链保留根因的失败对象。
        source: SharedError,
    },
}

impl fmt::Display for ContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(result) = self.fmt_application_runner(formatter) {
            return result;
        }
        if let Some(result) = self.fmt_scheduled_task(formatter) {
            return result;
        }
        if let Some(result) = self.fmt_event_listener(formatter) {
            return result;
        }
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
            Self::ApplicationRunnerDefinitionNotFound { .. }
            | Self::ApplicationRunnerScope { .. }
            | Self::DuplicateApplicationRunner { .. }
            | Self::ApplicationRunnerResolution { .. }
            | Self::ApplicationRunnerFailed { .. }
            | Self::ApplicationRunnerTimeout { .. } => {
                formatter.write_str("application runner error")
            }
            Self::ScheduledTaskDefinitionNotFound { .. }
            | Self::ScheduledTaskScope { .. }
            | Self::DuplicateScheduledTask { .. }
            | Self::ScheduledTaskResolution { .. } => formatter.write_str("scheduled task error"),
            Self::EventListenerDefinitionNotFound { .. }
            | Self::EventListenerScope { .. }
            | Self::DuplicateEventListener { .. } => {
                formatter.write_str("application event listener error")
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
            Self::PauseRestart {
                operation,
                component,
                phase,
                source,
            } => write!(
                formatter,
                "context {operation} failed: component {component} errored during {phase}: {source}"
            ),
        }
    }
}

impl ContextError {
    /// 格式化 Runner 专属错误；其他错误返回 `None` 交给主 Display 分支。
    fn fmt_application_runner(&self, formatter: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        match self {
            Self::ApplicationRunnerDefinitionNotFound { component } => Some(write!(
                formatter,
                "application runner component is not registered in IoC: {component}"
            )),
            Self::ApplicationRunnerScope { component, scope } => Some(write!(
                formatter,
                "application runner component {component} must be singleton, not {scope}"
            )),
            Self::DuplicateApplicationRunner { component } => Some(write!(
                formatter,
                "application runner component is registered more than once: {component}"
            )),
            Self::ApplicationRunnerResolution { component, .. } => Some(write!(
                formatter,
                "failed to resolve application runner {component}"
            )),
            Self::ApplicationRunnerFailed { source } => Some(write!(formatter, "{source}")),
            Self::ApplicationRunnerTimeout {
                runner,
                timeout,
                abort_settled,
            } => Some(write!(
                formatter,
                "application runner {runner} exceeded startup timeout {timeout:?}; \
                 Tokio abort settled: {abort_settled}"
            )),
            _ => None,
        }
    }

    /// 格式化周期任务声明和解析错误；其他错误交给主 Display 分支。
    fn fmt_scheduled_task(&self, formatter: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        match self {
            Self::ScheduledTaskDefinitionNotFound { component } => Some(write!(
                formatter,
                "scheduled task component is not registered in IoC: {component}"
            )),
            Self::ScheduledTaskScope { component, scope } => Some(write!(
                formatter,
                "scheduled task component {component} must be singleton, not {scope}"
            )),
            Self::DuplicateScheduledTask { component } => Some(write!(
                formatter,
                "scheduled task component is registered more than once: {component}"
            )),
            Self::ScheduledTaskResolution { component, .. } => Some(write!(
                formatter,
                "failed to resolve scheduled task {component}"
            )),
            _ => None,
        }
    }

    /// 格式化事件监听器声明错误；其他错误交给主 Display 分支。
    fn fmt_event_listener(&self, formatter: &mut fmt::Formatter<'_>) -> Option<fmt::Result> {
        match self {
            Self::EventListenerDefinitionNotFound { component, event } => Some(write!(
                formatter,
                "application event listener component is not registered in IoC: \
                 {component} for {event}"
            )),
            Self::EventListenerScope {
                component,
                event,
                scope,
            } => Some(write!(
                formatter,
                "application event listener component {component} for {event} must be \
                 singleton, not {scope}"
            )),
            Self::DuplicateEventListener { component, event } => Some(write!(
                formatter,
                "application event listener {component} is registered more than once for {event}"
            )),
            _ => None,
        }
    }
}

impl Error for ContextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ContainerWarmUp { source }
            | Self::ComponentResolution { source, .. }
            | Self::EventListenerResolution { source, .. }
            | Self::ApplicationRunnerResolution { source, .. }
            | Self::ScheduledTaskResolution { source, .. } => Some(source.as_ref()),
            Self::Lifecycle { source, .. }
            | Self::LifecycleCoordinator { source, .. }
            | Self::ShutdownSignal { source } => Some(source.as_ref()),
            Self::ApplicationRunnerFailed { source } => Some(source),
            Self::ManagedTask { source } => Some(source),
            _ => None,
        }
    }
}
