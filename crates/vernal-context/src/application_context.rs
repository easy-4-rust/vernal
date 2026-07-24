//! 应用上下文对象。

use std::{sync::Arc, time::Instant};

use tokio::runtime::Handle;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use vernal_aop::InvocationPlanCatalog;
use vernal_ioc::{ComponentKey, Container, ResolveError};

use crate::{
    ContextError, ContextState, DiagnosticOutcome, DiagnosticPhase, EventBus, Lifecycle,
    LifecyclePhase, StartupObservation, StartupReport,
    application_context_builder::LifecycleResolver, context_resources::ContextResources,
};

/// 组合 `IoC` 容器与 Tokio 生命周期状态机的应用上下文。
///
/// 所有状态转换由异步互斥锁串行化；组件按依赖顺序 initialize/start，按逆序
/// stop。关闭先触发共享取消令牌，再继续释放所有组件，即使某个 stop 失败。
pub struct ApplicationContext {
    container: Container,
    lifecycle_resolvers: Arc<[(ComponentKey, Arc<LifecycleResolver>)]>,
    components: Mutex<Vec<Arc<dyn Lifecycle>>>,
    state: RwLock<ContextState>,
    operation: Mutex<()>,
    resources: ContextResources,
    diagnostics: Mutex<StartupReport>,
}

impl ApplicationContext {
    /// 由建造器创建尚未 refresh 的上下文。
    pub(crate) fn new(
        container: Container,
        lifecycle_resolvers: Vec<(ComponentKey, Arc<LifecycleResolver>)>,
        resources: ContextResources,
    ) -> Self {
        let diagnostics = StartupReport::new(
            ContextState::Created.as_str().to_owned(),
            container.registry().snapshot(),
            resources.invocation_plans(),
            resources.diagnostics(),
        );
        Self {
            container,
            lifecycle_resolvers: lifecycle_resolvers.into(),
            components: Mutex::new(Vec::new()),
            state: RwLock::new(ContextState::Created),
            operation: Mutex::new(()),
            resources,
            diagnostics: Mutex::new(diagnostics),
        }
    }

    /// 预热单例、解析并初始化生命周期组件。
    ///
    /// # Errors
    ///
    /// 状态非法、组件解析失败或 initialize 失败时返回 [`ContextError`]。
    /// initialize 失败会逆序停止已成功初始化的组件并进入 Closed。
    pub async fn refresh(&self) -> Result<(), ContextError> {
        let _operation = self.operation.lock().await;
        self.require_state("refresh", ContextState::Created).await?;
        self.set_state(ContextState::Refreshing).await;

        let warm_up_started = Instant::now();
        if let Err(source) = self.container.warm_up() {
            self.record_observation(
                Self::resolve_failure_subject(&source),
                DiagnosticPhase::ContainerWarmUp,
                DiagnosticOutcome::Failed,
                warm_up_started,
            )
            .await;
            self.set_state(ContextState::Failed).await;
            return Err(ContextError::ContainerWarmUp {
                source: Box::new(source),
            });
        }
        self.record_observation(
            "vernal_ioc::Container",
            DiagnosticPhase::ContainerWarmUp,
            DiagnosticOutcome::Succeeded,
            warm_up_started,
        )
        .await;

        let mut initialized = Vec::with_capacity(self.lifecycle_resolvers.len());
        for (key, resolver) in self.lifecycle_resolvers.iter() {
            let resolution_started = Instant::now();
            let component = match resolver(&self.container) {
                Ok(component) => {
                    self.record_observation(
                        key.to_string(),
                        DiagnosticPhase::ComponentResolution,
                        DiagnosticOutcome::Succeeded,
                        resolution_started,
                    )
                    .await;
                    component
                }
                Err(source) => {
                    self.record_observation(
                        key.to_string(),
                        DiagnosticPhase::ComponentResolution,
                        DiagnosticOutcome::Failed,
                        resolution_started,
                    )
                    .await;
                    self.set_state(ContextState::RollingBack).await;
                    self.stop_all(&initialized).await;
                    self.set_state(ContextState::Closed).await;
                    return Err(ContextError::ComponentResolution {
                        component: key.clone(),
                        source: Box::new(source),
                    });
                }
            };
            let initialize_started = Instant::now();
            if let Err(source) = component.initialize().await {
                self.record_observation(
                    component.name(),
                    DiagnosticPhase::Initialize,
                    DiagnosticOutcome::Failed,
                    initialize_started,
                )
                .await;
                self.set_state(ContextState::RollingBack).await;
                let error = ContextError::Lifecycle {
                    component: component.name(),
                    phase: LifecyclePhase::Initialize,
                    source,
                };
                let _ = self.stop_component(&component).await;
                self.stop_all(&initialized).await;
                self.set_state(ContextState::Closed).await;
                return Err(error);
            }
            self.record_observation(
                component.name(),
                DiagnosticPhase::Initialize,
                DiagnosticOutcome::Succeeded,
                initialize_started,
            )
            .await;
            initialized.push(component);
        }

        *self.components.lock().await = initialized;
        self.set_state(ContextState::Refreshed).await;
        Ok(())
    }

    /// 启动全部生命周期组件并进入 Ready。
    ///
    /// # Errors
    ///
    /// 状态非法或任一 start 失败时返回 [`ContextError`]。失败会取消上下文并
    /// 逆序停止所有已初始化组件。
    pub async fn start(&self) -> Result<(), ContextError> {
        let _operation = self.operation.lock().await;
        self.require_state("start", ContextState::Refreshed).await?;
        self.set_state(ContextState::Starting).await;

        let components = self.components.lock().await.clone();
        for component in &components {
            let start_started = Instant::now();
            if let Err(source) = component.start(self.resources.cancellation().clone()).await {
                self.record_observation(
                    component.name(),
                    DiagnosticPhase::Start,
                    DiagnosticOutcome::Failed,
                    start_started,
                )
                .await;
                let error = ContextError::Lifecycle {
                    component: component.name(),
                    phase: LifecyclePhase::Start,
                    source,
                };
                self.resources.cancellation().cancel();
                self.set_state(ContextState::RollingBack).await;
                self.stop_all(&components).await;
                self.components.lock().await.clear();
                self.set_state(ContextState::Closed).await;
                return Err(error);
            }
            self.record_observation(
                component.name(),
                DiagnosticPhase::Start,
                DiagnosticOutcome::Succeeded,
                start_started,
            )
            .await;
        }

        self.set_state(ContextState::Ready).await;
        Ok(())
    }

    /// 幂等关闭上下文。
    ///
    /// # Errors
    ///
    /// 返回第一个 stop 错误，但仍会尝试关闭其余组件并最终进入 Closed。
    pub async fn close(&self) -> Result<(), ContextError> {
        let _operation = self.operation.lock().await;
        if self.state().await == ContextState::Closed {
            return Ok(());
        }

        self.resources.cancellation().cancel();
        let draining_state = if self.state().await == ContextState::Ready {
            ContextState::Draining
        } else {
            ContextState::RollingBack
        };
        self.set_state(draining_state).await;

        let components = std::mem::take(&mut *self.components.lock().await);
        let error = self.stop_all(&components).await;
        self.set_state(ContextState::Closed).await;
        error.map_or(Ok(()), Err)
    }

    /// 返回当前状态快照。
    pub async fn state(&self) -> ContextState {
        *self.state.read().await
    }

    /// 返回底层 `IoC` 容器。
    #[must_use]
    pub const fn container(&self) -> &Container {
        &self.container
    }

    /// 返回供组件和适配器派生子令牌的取消令牌。
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.resources.cancellation().clone()
    }

    /// 返回当前 Context 独占的类型化事件总线。
    #[must_use]
    pub fn events(&self) -> &EventBus {
        self.resources.events()
    }

    /// 返回高层应用建造器绑定的 Tokio Runtime Handle。
    ///
    /// 通过兼容性低层 API 创建的 Context 不隐式捕获 Runtime，因此返回
    /// `None`；通过 [`crate::VernalApplicationBuilder`] 创建时始终为 `Some`。
    #[must_use]
    pub fn runtime_handle(&self) -> Option<&Handle> {
        self.resources.runtime()
    }

    /// 返回应用构建阶段预编译的 AOP 调用计划目录。
    #[must_use]
    pub fn invocation_plans(&self) -> &InvocationPlanCatalog {
        self.resources.invocation_plans()
    }

    /// 返回调用时刻的只读、可序列化、脱敏启动报告。
    ///
    /// 返回值是拥有自身数据的快照；后续 start/close 操作只更新 Context 内部
    /// 报告，不会修改调用方已经取得的对象。
    pub async fn startup_report(&self) -> StartupReport {
        self.diagnostics.lock().await.clone()
    }

    /// 校验当前状态是否符合操作前置条件。
    async fn require_state(
        &self,
        operation: &'static str,
        expected: ContextState,
    ) -> Result<(), ContextError> {
        let state = self.state().await;
        if state == expected {
            Ok(())
        } else {
            Err(ContextError::InvalidState { operation, state })
        }
    }

    /// 原子替换可观察状态。
    async fn set_state(&self, state: ContextState) {
        *self.state.write().await = state;
        self.diagnostics
            .lock()
            .await
            .set_context_state(state.as_str().to_owned());
    }

    /// 执行单个组件停止钩子并记录脱敏结果。
    async fn stop_component(
        &self,
        component: &Arc<dyn Lifecycle>,
    ) -> Result<(), vernal_core::BoxError> {
        let started = Instant::now();
        let result = component.stop().await;
        let outcome = if result.is_ok() {
            DiagnosticOutcome::Succeeded
        } else {
            DiagnosticOutcome::Failed
        };
        self.record_observation(component.name(), DiagnosticPhase::Stop, outcome, started)
            .await;
        result
    }

    /// 逆序停止全部组件、记录每一步，并保留第一个错误。
    async fn stop_all(&self, components: &[Arc<dyn Lifecycle>]) -> Option<ContextError> {
        let mut first_error = None;
        for component in components.iter().rev() {
            let stop_error = self.stop_component(component).await.err();
            if first_error.is_none() {
                first_error = stop_error.map(|source| ContextError::Lifecycle {
                    component: component.name(),
                    phase: LifecyclePhase::Stop,
                    source,
                });
            }
        }
        first_error
    }

    /// 向内部报告追加一条不含错误正文的阶段记录。
    async fn record_observation(
        &self,
        subject: impl Into<String>,
        phase: DiagnosticPhase,
        outcome: DiagnosticOutcome,
        started: Instant,
    ) {
        let elapsed_microseconds = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
        self.diagnostics
            .lock()
            .await
            .record(StartupObservation::new(
                subject.into(),
                phase,
                outcome,
                elapsed_microseconds,
            ));
    }

    /// 从结构化解析错误中提取失败组件标识，不复制原始错误文本。
    fn resolve_failure_subject(error: &ResolveError) -> String {
        match error {
            ResolveError::Construction { component, .. }
            | ResolveError::TypeMismatch { component }
            | ResolveError::UndeclaredDependency { component, .. } => component.to_string(),
            ResolveError::TraitBindingTypeMismatch { target, .. } => target.to_string(),
            ResolveError::NotFound { component, .. }
            | ResolveError::Ambiguous { component, .. } => component.clone(),
            _ => "vernal_ioc::Container".to_owned(),
        }
    }
}
