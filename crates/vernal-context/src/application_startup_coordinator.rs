//! 应用上下文启动协调对象。

use std::{sync::Arc, time::Instant};

use tokio::{
    runtime::Handle,
    sync::oneshot,
    task::{JoinError, JoinHandle},
};
use vernal_beans::{ComponentKey, Container, ResolveError};

use crate::{
    ApplicationPausedEvent, ApplicationReadyEvent, ApplicationRefreshedEvent, ContextError,
    ContextState, DiagnosticOutcome, DiagnosticPhase, Lifecycle, LifecycleExecutionPolicy,
    LifecyclePhase, application_close_coordinator::ApplicationCloseCoordinator,
    application_context_builder::LifecycleResolver, lifecycle_task_executor::LifecycleTaskExecutor,
    managed_application_runner::ManagedApplicationRunner,
    managed_event_listener::ManagedEventListener, managed_scheduled_task::ManagedScheduledTask,
};

type OperationResult = Result<(), ContextError>;

/// 在独立 Tokio task 中串行执行 Context 的 refresh、start、Runner 与任务激活。
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
    application_runners: Arc<[ManagedApplicationRunner]>,
    scheduled_tasks: Arc<[ManagedScheduledTask]>,
    lifecycle: Arc<ApplicationCloseCoordinator>,
}

impl ApplicationStartupCoordinator {
    /// 创建绑定唯一 Container、解析计划和关闭协调器的启动对象。
    pub(crate) fn new(
        container: Arc<Container>,
        lifecycle_resolvers: Arc<[(ComponentKey, Arc<LifecycleResolver>)]>,
        event_listeners: Vec<ManagedEventListener>,
        application_runners: Vec<ManagedApplicationRunner>,
        scheduled_tasks: Vec<ManagedScheduledTask>,
        lifecycle: Arc<ApplicationCloseCoordinator>,
    ) -> Arc<Self> {
        Arc::new(Self {
            container,
            lifecycle_resolvers,
            event_listeners: Arc::from(event_listeners),
            application_runners: Arc::from(application_runners),
            scheduled_tasks: Arc::from(scheduled_tasks),
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
    /// Runtime 不可用、状态非法、组件启动/Runner 执行/周期任务激活失败，或协调
    /// 任务异常结束时返回结构化 [`ContextError`]。失败总会尝试逆序停止所有已
    /// 初始化组件。
    pub(crate) async fn start(self: &Arc<Self>) -> OperationResult {
        let handle = self.runtime_handle("start")?;
        let (sender, receiver) = oneshot::channel();
        let coordinator = Arc::clone(self);
        let operation = handle.spawn(async move { coordinator.finish_start().await });
        self.spawn_observer(&handle, "start", operation, sender);
        self.receive_result("start", receiver).await
    }

    /// 提交取消安全的 pause 操作并等待其结果。
    ///
    /// 对标 Spring 7.0 `ConfigurableApplicationContext#pause()` 与
    /// `LifecycleProcessor#onPause()`：只停止声明 `is_pauseable() == true` 的
    /// 生命周期组件；不可暂停组件保持运行。完成后 Context 进入 `Paused`
    /// 状态，并发布 [`ApplicationPausedEvent`]。
    ///
    /// # Errors
    ///
    /// Runtime 不可用、当前状态不是 `Ready`，或任一可暂停组件的 `pause()` 钩子
    /// 失败 / 超时 / panic 时返回结构化 [`ContextError`]。失败会尝试恢复到
    /// `Ready` 状态：未完成暂停的组件保持原状态，已暂停组件调用 `stop` 释放。
    pub(crate) async fn pause(self: &Arc<Self>) -> OperationResult {
        let handle = self.runtime_handle("pause")?;
        let (sender, receiver) = oneshot::channel();
        let coordinator = Arc::clone(self);
        let operation = handle.spawn(async move { coordinator.finish_pause().await });
        self.spawn_observer(&handle, "pause", operation, sender);
        self.receive_result("pause", receiver).await
    }

    /// 提交取消安全的 restart 操作并等待其结果。
    ///
    /// 对标 Spring 7.0 `ConfigurableApplicationContext#restart()` 与
    /// `LifecycleProcessor#onRestart()`：在 `Paused` 状态下重新调用可暂停组件
    /// 的 `start()` 钩子，恢复它们的工作循环。完成后 Context 推回 `Ready`
    /// 状态，并重新发布 [`ApplicationReadyEvent`]。
    ///
    /// # Errors
    ///
    /// Runtime 不可用、当前状态不是 `Paused`，或任一可暂停组件的 `start()` 钩子
    /// 失败时返回结构化 [`ContextError`]。失败会调用 `close()` 完成完整关闭，
    /// 与启动期失败处理一致。
    pub(crate) async fn restart(self: &Arc<Self>) -> OperationResult {
        let handle = self.runtime_handle("restart")?;
        let (sender, receiver) = oneshot::channel();
        let coordinator = Arc::clone(self);
        let operation = handle.spawn(async move { coordinator.finish_restart().await });
        self.spawn_observer(&handle, "restart", operation, sender);
        self.receive_result("restart", receiver).await
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
                "vernal_beans::Container",
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

    /// 在唯一操作锁内启动组件、执行 Runner、激活周期任务并提交 Ready。
    async fn finish_start(&self) -> OperationResult {
        let _operation = self.lifecycle.operation().lock().await;
        self.require_state("start", ContextState::Refreshed).await?;
        if self.is_cancelled() {
            let error = ContextError::LifecycleCancelled { operation: "start" };
            self.rollback_to_closed().await;
            return Err(error);
        }
        self.lifecycle.set_state(ContextState::Starting).await;

        // 对标 Spring 7.0 `DefaultLifecycleProcessor#startBeans(true)`：
        // - `is_auto_startup() == false` 的 Lifecycle 组件不参与自动启动
        //   （调用方应显式调用 `ApplicationContext::start` 完成该组件启动）
        // - 同拓扑深度的组件按 `phase()` 升序排列
        let components = self.lifecycle.components().await;
        let mut ordered: Vec<Arc<dyn Lifecycle>> = components
            .iter()
            .cloned()
            .filter(|component| component.is_auto_startup())
            .collect();
        ordered.sort_by_key(|component| component.phase());

        for component in &ordered {
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

        // Runner 是依赖图有序的一次性启动工作。它们共享生命周期 start 预算，
        // 但拥有独立错误语义；任何失败都在 Ready 提交前触发完整回滚。
        for runner in self.application_runners.iter() {
            if self.is_cancelled() {
                let error = ContextError::LifecycleCancelled { operation: "start" };
                self.rollback_to_closed().await;
                return Err(error);
            }
            let runner_started = Instant::now();
            let policy = *self.lifecycle.resources().lifecycle_execution_policy();
            let result = runner
                .execute(
                    &self.container,
                    self.lifecycle.resources().cancellation().child_token(),
                    policy,
                )
                .await;
            let outcome = if result.is_ok() {
                DiagnosticOutcome::Succeeded
            } else {
                DiagnosticOutcome::Failed
            };
            self.lifecycle
                .record_observation(
                    runner.component().to_string(),
                    DiagnosticPhase::ApplicationRunner,
                    outcome,
                    runner_started,
                )
                .await;
            if let Err(error) = result {
                self.lifecycle
                    .record_warning("context.application-runner.failed")
                    .await;
                self.rollback_to_closed().await;
                return Err(error);
            }
        }

        self.start_scheduled_tasks().await?;

        if self.is_cancelled() {
            let error = ContextError::LifecycleCancelled { operation: "start" };
            self.rollback_to_closed().await;
            return Err(error);
        }
        self.lifecycle.set_state(ContextState::Ready).await;
        // Ready 事件只在全部 start、Runner、周期任务激活与取消检查通过后发布。
        // 后台任务运行期失败会由监督器取消应用，不反向改写已经提交的状态事实。
        let _delivered = self
            .lifecycle
            .resources()
            .events()
            .publish(ApplicationReadyEvent::new())
            .await;
        Ok(())
    }

    /// 按依赖图顺序把周期任务提交给 Context-local Tokio 监督器。
    ///
    /// 激活顺序确定，但不同任务提交后可以并发运行。这里只承诺监督器已经接受
    /// 任务；需要首次执行成功才能就绪的工作必须使用 `ApplicationRunner`。
    async fn start_scheduled_tasks(&self) -> OperationResult {
        let Some(managed_tasks) = self.lifecycle.resources().managed_tasks().cloned() else {
            if self.scheduled_tasks.is_empty() {
                return Ok(());
            }
            let error = ContextError::LifecycleCoordinator {
                operation: "scheduled-task-activation",
                source: Arc::new(std::io::Error::other(
                    "scheduled tasks require a managed Tokio application context",
                )),
            };
            self.rollback_to_closed().await;
            return Err(error);
        };

        for task in self.scheduled_tasks.iter() {
            if self.is_cancelled() {
                let error = ContextError::LifecycleCancelled { operation: "start" };
                self.rollback_to_closed().await;
                return Err(error);
            }
            let started = Instant::now();
            let result = task.start(&self.container, Arc::clone(&managed_tasks));
            let outcome = if result.is_ok() {
                DiagnosticOutcome::Succeeded
            } else {
                DiagnosticOutcome::Failed
            };
            self.lifecycle
                .record_observation(
                    task.component().to_string(),
                    DiagnosticPhase::ScheduledTaskActivation,
                    outcome,
                    started,
                )
                .await;
            if let Err(error) = result {
                self.lifecycle
                    .record_warning("context.scheduled-task.activation-failed")
                    .await;
                self.rollback_to_closed().await;
                return Err(error);
            }
        }
        Ok(())
    }

    /// 在有界独立任务中执行 initialize，并隔离业务错误、panic 与永久等待。
    async fn finish_pause(&self) -> OperationResult {
        let _operation = self.lifecycle.operation().lock().await;
        self.require_state("pause", ContextState::Ready).await?;
        if self.is_cancelled() {
            return Err(ContextError::LifecycleCancelled { operation: "pause" });
        }
        self.lifecycle.set_state(ContextState::Pausing).await;

        // 只暂停声明 is_pauseable() == true 的组件；其余组件保持运行。
        // 对标 Spring `DefaultLifecycleProcessor.stopBeans(true)`：调用方
        // 传入 `pauseableOnly=true` 让 stop 链只触及 SmartLifecycle 组件。
        let components = self.lifecycle.components().await;
        let pauseable: Vec<Arc<dyn Lifecycle>> = components
            .iter()
            .cloned()
            .filter(|component| component.is_pauseable())
            .collect();
        if let Some(error) = self.lifecycle.pause_all(&pauseable).await {
            // 暂停失败：保持未暂停组件运行，已暂停组件按 stop 释放，最终关闭。
            // 与 Spring `DefaultLifecycleProcessor` 行为一致 —— 暂停阶段异常会
            // 让整个生命周期协调器进入失败态。
            self.lifecycle
                .record_warning("context.lifecycle-hook.pause-failed")
                .await;
            self.rollback_to_closed().await;
            return Self::upgrade_pause_error("pause", error);
        }
        self.lifecycle.set_state(ContextState::Paused).await;
        let _delivered = self
            .lifecycle
            .resources()
            .events()
            .publish(ApplicationPausedEvent::new())
            .await;
        Ok(())
    }

    /// 在有界独立任务中重新启动已暂停的可暂停组件，并提交 Ready。
    async fn finish_restart(&self) -> OperationResult {
        let _operation = self.lifecycle.operation().lock().await;
        self.require_state("restart", ContextState::Paused).await?;
        if self.is_cancelled() {
            return Err(ContextError::LifecycleCancelled {
                operation: "restart",
            });
        }
        self.lifecycle.set_state(ContextState::Starting).await;

        // 只重启此前已暂停的可暂停组件；不可暂停组件从未停止，不需要重启。
        // 对标 Spring `DefaultLifecycleProcessor.onRestart()`：先 stop 已运行的
        // pauseable bean，再 startBeans(true)；vernal 已经在 pause 阶段停止过
        // 可暂停组件，restart 直接重新 start 即可。
        let components = self.lifecycle.components().await;
        for component in &components {
            if !component.is_pauseable() {
                continue;
            }
            if self.is_cancelled() {
                let error = ContextError::LifecycleCancelled {
                    operation: "restart",
                };
                self.rollback_to_closed().await;
                return Err(error);
            }
            let start_started = Instant::now();
            let policy = *self.lifecycle.resources().lifecycle_execution_policy();
            let result = Self::start_component(
                Arc::clone(component),
                self.lifecycle.resources().cancellation().clone(),
                policy,
            )
            .await;
            let outcome = if result.is_ok() {
                DiagnosticOutcome::Succeeded
            } else {
                DiagnosticOutcome::Failed
            };
            self.lifecycle
                .record_observation(
                    component.name(),
                    DiagnosticPhase::Start,
                    outcome,
                    start_started,
                )
                .await;
            if let Err(error) = result {
                self.lifecycle.record_lifecycle_warning(&error).await;
                self.rollback_to_closed().await;
                return Self::upgrade_pause_error("restart", error);
            }
        }

        self.lifecycle.set_state(ContextState::Ready).await;
        let _delivered = self
            .lifecycle
            .resources()
            .events()
            .publish(ApplicationReadyEvent::new())
            .await;
        Ok(())
    }

    /// 把任意 [`ContextError`] 升级为 [`ContextError::PauseRestart`]，保留原始
    /// 错误链；非生命周期错误原样返回。
    fn upgrade_pause_error(operation: &'static str, error: ContextError) -> OperationResult {
        match error {
            ContextError::Lifecycle {
                component,
                phase,
                source,
            } => Err(ContextError::PauseRestart {
                operation,
                component,
                phase,
                source,
            }),
            ContextError::LifecycleTimeout {
                component,
                phase,
                timeout,
                abort_settled,
            } => {
                // 超时错误也升级到 PauseRestart，但需要重新包装源错误。
                // LifecycleTimeout 不携带 source，我们重建一个 io::Error 作为
                // 诊断占位，保留可读的错误信息。
                let placeholder = std::io::Error::other(format!(
                    "{operation} phase {phase} for component {component} exceeded {timeout:?}; \
                     abort settled: {abort_settled}"
                ));
                Err(ContextError::PauseRestart {
                    operation,
                    component,
                    phase,
                    source: Arc::new(placeholder),
                })
            }
            other => Err(other),
        }
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
            _ => "vernal_beans::Container".to_owned(),
        }
    }
}
