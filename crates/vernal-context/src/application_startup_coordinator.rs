//! 应用上下文启动协调对象。

use std::{sync::Arc, time::Instant};

use tokio::{
    runtime::Handle,
    sync::oneshot,
    task::{JoinError, JoinHandle},
};
use vernal_ioc::{ComponentKey, Container, ResolveError};

use crate::{
    ApplicationReadyEvent, ApplicationRefreshedEvent, ContextError, ContextState,
    DiagnosticOutcome, DiagnosticPhase, Lifecycle, LifecycleExecutionPolicy, LifecyclePhase,
    application_close_coordinator::ApplicationCloseCoordinator,
    application_context_builder::LifecycleResolver, lifecycle_task_executor::LifecycleTaskExecutor,
    managed_event_listener::ManagedEventListener,
};

type OperationResult = Result<(), ContextError>;

/// 在独立 Tokio task 中串行执行 Context 的 refresh 与 start 阶段。
///
/// 每次公开调用只等待一个一次性结果通道，真正的生命周期阶段由本对象拥有的任务
/// 执行。调用方丢弃 `refresh()` 或 `start()` Future 时，结果接收端会被丢弃，但
/// 操作任务和观察任务仍继续：成功时提交目标状态，失败或 panic 时取消应用、逆序
/// 停止已经进入组件栈的对象并到达确定终态。
///
/// 该设计保留 tx-di “显式初始化、异步启动、关闭”阶段的可读性，但不把阶段所有权
/// 交给临时调用者，也不使用进程级 App 或全局组件表。
pub(crate) struct ApplicationStartupCoordinator {
    container: Arc<Container>,
    lifecycle_resolvers: Arc<[(ComponentKey, Arc<LifecycleResolver>)]>,
    event_listeners: Arc<[ManagedEventListener]>,
    lifecycle: Arc<ApplicationCloseCoordinator>,
}

impl ApplicationStartupCoordinator {
    /// 创建绑定唯一 Container、解析计划和关闭协调器的启动对象。
    pub(crate) fn new(
        container: Arc<Container>,
        lifecycle_resolvers: Arc<[(ComponentKey, Arc<LifecycleResolver>)]>,
        event_listeners: Vec<ManagedEventListener>,
        lifecycle: Arc<ApplicationCloseCoordinator>,
    ) -> Arc<Self> {
        Arc::new(Self {
            container,
            lifecycle_resolvers,
            event_listeners: Arc::from(event_listeners),
            lifecycle,
        })
    }

    /// 返回当前应用唯一的组件容器。
    pub(crate) fn container(&self) -> &Container {
        &self.container
    }

    /// 返回与启动阶段共享状态、组件栈和资源的关闭协调器。
    pub(crate) const fn lifecycle(&self) -> &Arc<ApplicationCloseCoordinator> {
        &self.lifecycle
    }

    /// 提交取消安全的 refresh 操作并等待其结果。
    ///
    /// # Errors
    ///
    /// Runtime 不可用、状态非法、组件预热/解析/初始化失败，或协调任务异常结束时
    /// 返回结构化 [`ContextError`]。
    pub(crate) async fn refresh(self: &Arc<Self>) -> OperationResult {
        let handle = self.runtime_handle("refresh")?;
        let (sender, receiver) = oneshot::channel();
        let coordinator = Arc::clone(self);
        let operation = handle.spawn(async move { coordinator.finish_refresh().await });
        self.spawn_observer(&handle, "refresh", operation, sender);
        self.receive_result("refresh", receiver).await
    }

    /// 提交取消安全的 start 操作并等待其结果。
    ///
    /// # Errors
    ///
    /// Runtime 不可用、状态非法、组件启动失败，或协调任务异常结束时返回结构化
    /// [`ContextError`]。失败总会尝试逆序停止所有已初始化组件。
    pub(crate) async fn start(self: &Arc<Self>) -> OperationResult {
        let handle = self.runtime_handle("start")?;
        let (sender, receiver) = oneshot::channel();
        let coordinator = Arc::clone(self);
        let operation = handle.spawn(async move { coordinator.finish_start().await });
        self.spawn_observer(&handle, "start", operation, sender);
        self.receive_result("start", receiver).await
    }

    /// 在唯一操作锁内完成容器预热、组件解析与顺序初始化。
    async fn finish_refresh(&self) -> OperationResult {
        let _operation = self.lifecycle.operation().lock().await;
        self.require_state("refresh", ContextState::Created).await?;
        if self.is_cancelled() {
            let error = ContextError::LifecycleCancelled {
                operation: "refresh",
            };
            self.rollback_to_closed().await;
            return Err(error);
        }
        self.lifecycle.set_state(ContextState::Refreshing).await;

        self.warm_up_container().await?;
        self.start_event_listeners().await?;

        for (key, resolver) in self.lifecycle_resolvers.iter() {
            if self.is_cancelled() {
                let error = ContextError::LifecycleCancelled {
                    operation: "refresh",
                };
                self.rollback_to_closed().await;
                return Err(error);
            }
            self.resolve_and_initialize(key, resolver).await?;
        }

        if self.is_cancelled() {
            let error = ContextError::LifecycleCancelled {
                operation: "refresh",
            };
            self.rollback_to_closed().await;
            return Err(error);
        }
        self.lifecycle.set_state(ContextState::Refreshed).await;
        // 状态先提交再发布事实事件。监听器已经在 initialize 前建立订阅，但事件
        // 处理属于受管后台任务，refresh 不把异步消费伪装成同步完成屏障。
        let _delivered = self
            .lifecycle
            .resources()
            .events()
            .publish(ApplicationRefreshedEvent::new())
            .await;
        Ok(())
    }

    /// 在任何生命周期 initialize 之前建立全部强类型事件订阅。
    ///
    /// 注册顺序已经由 `IoC` 依赖计划稳定排序。任一解析或任务提交失败都会取消应用，
    /// 排空此前启动的监听任务并让 refresh 进入确定的 Closed 终态。
    async fn start_event_listeners(&self) -> OperationResult {
        let Some(managed_tasks) = self.lifecycle.resources().managed_tasks().cloned() else {
            // 只有高层 VernalApplicationBuilder 能登记事件监听器；此分支保护内部
            // 不变量，避免未来低层 Context 误装配后 panic。
            if self.event_listeners.is_empty() {
                return Ok(());
            }
            let error = ContextError::LifecycleCoordinator {
                operation: "event-listener-start",
                source: Arc::new(std::io::Error::other(
                    "managed event listeners require a managed Tokio application context",
                )),
            };
            self.rollback_to_closed().await;
            return Err(error);
        };
        let events = Arc::clone(self.lifecycle.resources().events_arc());
        for listener in self.event_listeners.iter() {
            if let Err(error) = listener
                .start(
                    &self.container,
                    Arc::clone(&events),
                    Arc::clone(&managed_tasks),
                )
                .await
            {
                self.rollback_to_closed().await;
                return Err(error);
            }
        }
        Ok(())
    }

    /// 预热单例容器并记录不包含业务错误正文的诊断结果。
    async fn warm_up_container(&self) -> OperationResult {
        let started = Instant::now();
        if let Err(source) = self.container.warm_up() {
            self.lifecycle
                .record_observation(
                    Self::resolve_failure_subject(&source),
                    DiagnosticPhase::ContainerWarmUp,
                    DiagnosticOutcome::Failed,
                    started,
                )
                .await;
            self.lifecycle.resources().cancellation().cancel();
            let _ = self.lifecycle.shutdown_managed_tasks().await;
            self.lifecycle.set_state(ContextState::Failed).await;
            return Err(ContextError::ContainerWarmUp {
                source: Box::new(source),
            });
        }
        self.lifecycle
            .record_observation(
                "vernal_ioc::Container",
                DiagnosticPhase::ContainerWarmUp,
                DiagnosticOutcome::Succeeded,
                started,
            )
            .await;
        Ok(())
    }

    /// 解析一个生命周期定义、先登记共享所有权，再执行 initialize。
    async fn resolve_and_initialize(
        &self,
        key: &ComponentKey,
        resolver: &Arc<LifecycleResolver>,
    ) -> OperationResult {
        let resolution_started = Instant::now();
        let component = match resolver(&self.container) {
            Ok(component) => {
                self.lifecycle
                    .record_observation(
                        key.to_string(),
                        DiagnosticPhase::ComponentResolution,
                        DiagnosticOutcome::Succeeded,
                        resolution_started,
                    )
                    .await;
                component
            }
            Err(source) => {
                self.lifecycle
                    .record_observation(
                        key.to_string(),
                        DiagnosticPhase::ComponentResolution,
                        DiagnosticOutcome::Failed,
                        resolution_started,
                    )
                    .await;
                let error = ContextError::ComponentResolution {
                    component: key.clone(),
                    source: Box::new(source),
                };
                self.rollback_to_closed().await;
                return Err(error);
            }
        };

        // 先把当前组件压入共享栈，再执行 initialize。即使用户 Future panic，
        // 外层观察任务也能从共享栈找到它并完成 stop，而不是只让 Arc 被动 Drop。
        self.lifecycle.push_component(Arc::clone(&component)).await;
        let initialize_started = Instant::now();
        let policy = *self.lifecycle.resources().lifecycle_execution_policy();
        if let Err(error) = Self::initialize_component(Arc::clone(&component), policy).await {
            self.lifecycle
                .record_observation(
                    component.name(),
                    DiagnosticPhase::Initialize,
                    DiagnosticOutcome::Failed,
                    initialize_started,
                )
                .await;
            self.lifecycle.record_lifecycle_warning(&error).await;
            self.rollback_to_closed().await;
            return Err(error);
        }
        self.lifecycle
            .record_observation(
                component.name(),
                DiagnosticPhase::Initialize,
                DiagnosticOutcome::Succeeded,
                initialize_started,
            )
            .await;
        Ok(())
    }

    /// 在唯一操作锁内按依赖顺序启动全部已初始化组件。
    async fn finish_start(&self) -> OperationResult {
        let _operation = self.lifecycle.operation().lock().await;
        self.require_state("start", ContextState::Refreshed).await?;
        if self.is_cancelled() {
            let error = ContextError::LifecycleCancelled { operation: "start" };
            self.rollback_to_closed().await;
            return Err(error);
        }
        self.lifecycle.set_state(ContextState::Starting).await;

        let components = self.lifecycle.components().await;
        for component in &components {
            if self.is_cancelled() {
                let error = ContextError::LifecycleCancelled { operation: "start" };
                self.rollback_to_closed().await;
                return Err(error);
            }
            let start_started = Instant::now();
            let policy = *self.lifecycle.resources().lifecycle_execution_policy();
            if let Err(error) = Self::start_component(
                Arc::clone(component),
                self.lifecycle.resources().cancellation().clone(),
                policy,
            )
            .await
            {
                self.lifecycle
                    .record_observation(
                        component.name(),
                        DiagnosticPhase::Start,
                        DiagnosticOutcome::Failed,
                        start_started,
                    )
                    .await;
                self.lifecycle.record_lifecycle_warning(&error).await;
                self.rollback_to_closed().await;
                return Err(error);
            }
            self.lifecycle
                .record_observation(
                    component.name(),
                    DiagnosticPhase::Start,
                    DiagnosticOutcome::Succeeded,
                    start_started,
                )
                .await;
        }

        if self.is_cancelled() {
            let error = ContextError::LifecycleCancelled { operation: "start" };
            self.rollback_to_closed().await;
            return Err(error);
        }
        self.lifecycle.set_state(ContextState::Ready).await;
        // Ready 事件只在全部 start 成功且取消检查通过后发布。监听器失败会稍后由
        // 任务监督器取消应用，不反向改写已经提交的状态转换结果。
        let _delivered = self
            .lifecycle
            .resources()
            .events()
            .publish(ApplicationReadyEvent::new())
            .await;
        Ok(())
    }

    /// 在有界独立任务中执行 initialize，并隔离业务错误、panic 与永久等待。
    async fn initialize_component(
        component: Arc<dyn Lifecycle>,
        policy: LifecycleExecutionPolicy,
    ) -> OperationResult {
        let name = component.name();
        let task = tokio::spawn(async move { component.initialize().await });
        LifecycleTaskExecutor::execute(
            task,
            name,
            LifecyclePhase::Initialize,
            policy.initialize_timeout(),
            policy.abort_timeout(),
        )
        .await
    }

    /// 在有界独立任务中执行 start，并在超时后请求 Tokio abort。
    async fn start_component(
        component: Arc<dyn Lifecycle>,
        cancellation: tokio_util::sync::CancellationToken,
        policy: LifecycleExecutionPolicy,
    ) -> OperationResult {
        let name = component.name();
        let task = tokio::spawn(async move { component.start(cancellation).await });
        LifecycleTaskExecutor::execute(
            task,
            name,
            LifecyclePhase::Start,
            policy.start_timeout(),
            policy.abort_timeout(),
        )
        .await
    }

    /// 取消应用并逆序释放共享组件栈，最后发布 Closed。
    async fn rollback_to_closed(&self) {
        self.lifecycle.resources().cancellation().cancel();
        self.lifecycle.set_state(ContextState::RollingBack).await;
        let _ = self.lifecycle.shutdown_managed_tasks().await;
        let components = self.lifecycle.take_components().await;
        let _ = self.lifecycle.stop_all(&components).await;
        self.lifecycle.set_state(ContextState::Closed).await;
    }

    /// 为操作创建一个始终存活到 Join 结果完成的观察任务。
    fn spawn_observer(
        self: &Arc<Self>,
        handle: &Handle,
        operation: &'static str,
        task: JoinHandle<OperationResult>,
        sender: oneshot::Sender<OperationResult>,
    ) {
        let coordinator = Arc::clone(self);
        handle.spawn(async move {
            let result = coordinator.observe_operation(operation, task).await;
            // 接收端被取消只表示调用方不再观察；阶段任务已经完整结束，发送失败
            // 不得改变 Context 的状态或重新执行回滚。
            let _ = sender.send(result);
        });
    }

    /// 消费阶段 JoinHandle，并在协调任务自身异常结束时执行兜底关闭。
    async fn observe_operation(
        &self,
        operation: &'static str,
        task: JoinHandle<OperationResult>,
    ) -> OperationResult {
        match task.await {
            Ok(result) => result,
            Err(source) => {
                self.lifecycle
                    .record_warning("context.lifecycle-coordinator.failed")
                    .await;
                self.lifecycle.resources().cancellation().cancel();
                let error = Self::coordinator_error(operation, source);
                let _ = self.lifecycle.close().await;
                Err(error)
            }
        }
    }

    /// 等待一次性结果；观察任务异常消失时返回结构化协调错误。
    async fn receive_result(
        &self,
        operation: &'static str,
        receiver: oneshot::Receiver<OperationResult>,
    ) -> OperationResult {
        receiver
            .await
            .map_err(|source| ContextError::LifecycleCoordinator {
                operation,
                source: Arc::new(source),
            })?
    }

    /// 获取构建阶段绑定的 Runtime，或低层 Context 调用现场的 Runtime。
    fn runtime_handle(&self, operation: &'static str) -> Result<Handle, ContextError> {
        self.lifecycle
            .resources()
            .runtime()
            .cloned()
            .or_else(|| Handle::try_current().ok())
            .ok_or_else(|| ContextError::LifecycleCoordinator {
                operation,
                source: Arc::new(std::io::Error::other(
                    "Tokio runtime is unavailable for context lifecycle operation",
                )),
            })
    }

    /// 校验阶段前置状态，防止重复执行不可重入钩子。
    async fn require_state(
        &self,
        operation: &'static str,
        expected: ContextState,
    ) -> OperationResult {
        let state = self.lifecycle.state().await;
        if state == expected {
            Ok(())
        } else {
            Err(ContextError::InvalidState { operation, state })
        }
    }

    /// 返回应用取消树是否已经收到关闭或任务失败信号。
    fn is_cancelled(&self) -> bool {
        self.lifecycle.resources().cancellation().is_cancelled()
    }

    /// 把协调任务 panic 或异常取消转为稳定操作错误。
    fn coordinator_error(operation: &'static str, source: JoinError) -> ContextError {
        ContextError::LifecycleCoordinator {
            operation,
            source: Arc::new(source),
        }
    }

    /// 从结构化解析错误提取组件标识，不复制底层业务错误文本。
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
