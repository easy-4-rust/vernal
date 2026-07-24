//! 应用上下文对象。

use std::{sync::Arc, time::Instant};

use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;
use vernal_aop::InvocationPlanCatalog;
use vernal_ioc::{ComponentKey, Container, ResolveError, ScopeContext};

use crate::{
    ContextError, ContextState, DiagnosticOutcome, DiagnosticPhase, EventBus, Lifecycle,
    LifecyclePhase, ManagedTaskError, ManagedTaskSupervisor, StartupReport, TaskShutdownPolicy,
    application_close_coordinator::ApplicationCloseCoordinator,
    application_context_builder::LifecycleResolver, context_resources::ContextResources,
};

/// 组合 `IoC` 容器与 Tokio 生命周期状态机的应用上下文。
///
/// 所有状态转换由异步互斥锁串行化；组件按依赖顺序 initialize/start，按逆序
/// stop。关闭先触发共享取消令牌并排空受管 Tokio 任务，再继续释放所有组件，
/// 即使任务或某个 stop 失败也会推进到 Closed。
pub struct ApplicationContext {
    container: Container,
    lifecycle_resolvers: Arc<[(ComponentKey, Arc<LifecycleResolver>)]>,
    close_coordinator: Arc<ApplicationCloseCoordinator>,
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
            resources.local_invocation_plans(),
            resources.diagnostics(),
        );
        let close_coordinator = ApplicationCloseCoordinator::new(resources, diagnostics);
        Self {
            container,
            lifecycle_resolvers: lifecycle_resolvers.into(),
            close_coordinator,
        }
    }

    /// 预热单例、解析并初始化生命周期组件。
    ///
    /// # Errors
    ///
    /// 状态非法、组件解析失败或 initialize 失败时返回 [`ContextError`]。
    /// initialize 失败会逆序停止已成功初始化的组件并进入 Closed。
    pub async fn refresh(&self) -> Result<(), ContextError> {
        let _operation = self.close_coordinator.operation().lock().await;
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
            self.close_coordinator.resources().cancellation().cancel();
            let _ = self.shutdown_managed_tasks().await;
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
                    self.close_coordinator.resources().cancellation().cancel();
                    self.set_state(ContextState::RollingBack).await;
                    let _ = self.shutdown_managed_tasks().await;
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
                    source: source.into(),
                };
                self.close_coordinator.resources().cancellation().cancel();
                let _ = self.shutdown_managed_tasks().await;
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

        self.close_coordinator.replace_components(initialized).await;
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
        let _operation = self.close_coordinator.operation().lock().await;
        self.require_state("start", ContextState::Refreshed).await?;
        self.set_state(ContextState::Starting).await;

        let components = self.close_coordinator.components().await;
        for component in &components {
            let start_started = Instant::now();
            if let Err(source) = component
                .start(self.close_coordinator.resources().cancellation().clone())
                .await
            {
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
                    source: source.into(),
                };
                self.close_coordinator.resources().cancellation().cancel();
                self.set_state(ContextState::RollingBack).await;
                let _ = self.shutdown_managed_tasks().await;
                self.stop_all(&components).await;
                self.close_coordinator.clear_components().await;
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
    /// 返回第一个受管任务或 stop 错误，但仍会尝试关闭其余组件并最终进入 Closed。
    /// 关闭由唯一 Tokio 协调任务持有；丢弃当前调用者的 Future 不会中断资源释放，
    /// 后续或并发调用者会等待同一个可克隆结果。
    pub async fn close(&self) -> Result<(), ContextError> {
        self.close_coordinator.close().await
    }

    /// 等待应用取消信号，并在收到信号后执行取消安全的完整关闭。
    ///
    /// 受管任务返回错误、发生 panic 或被异常取消时会取消应用令牌，因此服务主函数
    /// 可以在 `start()` 成功后等待本方法，把后台任务失败转换成 Context 关闭结果。
    /// 外部信号处理器也可以调用 [`CancellationToken::cancel`] 复用同一条关闭路径。
    ///
    /// # Errors
    ///
    /// 返回 [`Self::close`] 的第一个受管任务、组件停止或协调器错误。
    pub async fn run_until_cancelled(&self) -> Result<(), ContextError> {
        let cancellation = self.cancellation_token();
        cancellation.cancelled().await;
        self.close().await
    }

    /// 返回当前状态快照。
    pub async fn state(&self) -> ContextState {
        self.close_coordinator.state().await
    }

    /// 返回底层 `IoC` 容器。
    #[must_use]
    pub const fn container(&self) -> &Container {
        &self.container
    }

    /// 进入绑定当前应用和类型标记 `S` 的根自定义作用域。
    ///
    /// Scope 使用应用取消令牌的子令牌：应用关闭会立即阻止新组件解析，但 Scope
    /// 所有者仍须显式调用 [`ScopeContext::close`] 执行自己的异步关闭钩子。
    #[must_use]
    pub fn open_scope<S>(&self) -> Arc<ScopeContext>
    where
        S: 'static,
    {
        self.container.open_scope_with_cancellation::<S>(
            self.close_coordinator
                .resources()
                .cancellation()
                .child_token(),
        )
    }

    /// 返回供组件和适配器派生子令牌的取消令牌。
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.close_coordinator.resources().cancellation().clone()
    }

    /// 返回高层应用建造器创建的 Tokio 任务监督器。
    ///
    /// 低层 `ApplicationContextBuilder` 不捕获 Runtime，因此返回 `None`。高层
    /// 路径返回的对象与注册到 `IoC` 的 `Arc<ManagedTaskSupervisor>` 是同一实例。
    #[must_use]
    pub fn managed_tasks(&self) -> Option<&Arc<ManagedTaskSupervisor>> {
        self.close_coordinator.resources().managed_tasks()
    }

    /// 返回受管任务的两阶段停机策略。
    #[must_use]
    pub fn task_shutdown_policy(&self) -> &TaskShutdownPolicy {
        self.close_coordinator.resources().task_shutdown_policy()
    }

    /// 返回当前 Context 独占的类型化事件总线。
    #[must_use]
    pub fn events(&self) -> &EventBus {
        self.close_coordinator.resources().events()
    }

    /// 返回应用作用域共用的异步清理策略。
    ///
    /// 高层建造器默认提供 30 秒上限；调用方可以在构建阶段显式改为其他上限或
    /// 无界等待。返回借用保证运行期间策略不可漂移。
    #[must_use]
    pub fn scope_cleanup_policy(&self) -> &crate::ScopeCleanupPolicy {
        self.close_coordinator.resources().scope_cleanup_policy()
    }

    /// 返回高层应用建造器绑定的 Tokio Runtime Handle。
    ///
    /// 通过兼容性低层 API 创建的 Context 不隐式捕获 Runtime，因此返回
    /// `None`；通过 [`crate::VernalApplicationBuilder`] 创建时始终为 `Some`。
    #[must_use]
    pub fn runtime_handle(&self) -> Option<&Handle> {
        self.close_coordinator.resources().runtime()
    }

    /// 返回应用构建阶段预编译的 AOP 调用计划目录。
    #[must_use]
    pub fn invocation_plans(&self) -> &InvocationPlanCatalog {
        self.close_coordinator.resources().invocation_plans()
    }

    /// 返回应用构建阶段预编译的 Local-AOP 调用计划目录。
    #[must_use]
    pub fn local_invocation_plans(&self) -> &vernal_aop::LocalInvocationPlanCatalog {
        self.close_coordinator.resources().local_invocation_plans()
    }

    /// 返回调用时刻的只读、可序列化、脱敏启动报告。
    ///
    /// 返回值是拥有自身数据的快照；后续 start/close 操作只更新 Context 内部
    /// 报告，不会修改调用方已经取得的对象。
    pub async fn startup_report(&self) -> StartupReport {
        self.close_coordinator.startup_report().await
    }

    /// 记录一条不携带运行时数据的 Context-local 脱敏告警代码。
    ///
    /// 该入口只接受静态字符串，从类型层阻止请求参数、凭证、数据库错误正文或其他
    /// 敏感值进入可序列化诊断。相同代码自动去重，多个应用 Context 之间互不共享。
    pub async fn record_runtime_warning(&self, warning: &'static str) {
        self.close_coordinator.record_warning(warning).await;
    }

    /// 停止应用监督器并记录不含任务错误正文的静态诊断代码。
    async fn shutdown_managed_tasks(&self) -> Option<ManagedTaskError> {
        self.close_coordinator.shutdown_managed_tasks().await
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
        self.close_coordinator.set_state(state).await;
    }

    /// 执行单个组件停止钩子并记录脱敏结果。
    async fn stop_component(
        &self,
        component: &Arc<dyn Lifecycle>,
    ) -> Result<(), vernal_core::SharedError> {
        self.close_coordinator.stop_component(component).await
    }

    /// 逆序停止全部组件、记录每一步，并保留第一个错误。
    async fn stop_all(&self, components: &[Arc<dyn Lifecycle>]) -> Option<ContextError> {
        self.close_coordinator.stop_all(components).await
    }

    /// 向内部报告追加一条不含错误正文的阶段记录。
    async fn record_observation(
        &self,
        subject: impl Into<String>,
        phase: DiagnosticPhase,
        outcome: DiagnosticOutcome,
        started: Instant,
    ) {
        self.close_coordinator
            .record_observation(subject, phase, outcome, started)
            .await;
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
