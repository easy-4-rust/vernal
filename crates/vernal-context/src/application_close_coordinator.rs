//! 应用上下文关闭协调对象。

use std::{
    mem,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use tokio::{
    runtime::Handle,
    sync::{Mutex, RwLock, watch},
};

use crate::{
    ContextError, ContextState, DiagnosticOutcome, DiagnosticPhase, Lifecycle, LifecyclePhase,
    ManagedTaskError, StartupObservation, StartupReport, context_resources::ContextResources,
    lifecycle_task_executor::LifecycleTaskExecutor,
};

type CloseResult = Result<(), ContextError>;

/// 持有 `ApplicationContext` 可变生命周期状态并执行唯一的后台关闭流程。
///
/// Context 本身是面向调用方的门面，本对象才是关闭状态、组件栈、诊断报告和停机
/// 结果的唯一所有者。第一次 `close` 会把关闭流程提交给 Tokio；调用方取消等待只会
/// 丢弃自己的观察 Future，不会丢弃正在运行的组件停止流程。后续和并发调用者通过
/// `watch` 订阅同一份可克隆结构化结果，因而不会重复调用组件的 `stop`。
pub(crate) struct ApplicationCloseCoordinator {
    components: Mutex<Vec<Arc<dyn Lifecycle>>>,
    state: RwLock<ContextState>,
    operation: Mutex<()>,
    resources: ContextResources,
    diagnostics: Mutex<StartupReport>,
    close_started: AtomicBool,
    close_result: watch::Sender<Option<CloseResult>>,
}

impl ApplicationCloseCoordinator {
    /// 创建尚未开始关闭的 Context 生命周期协调器。
    pub(crate) fn new(resources: ContextResources, diagnostics: StartupReport) -> Arc<Self> {
        let (close_result, _) = watch::channel(None);
        Arc::new(Self {
            components: Mutex::new(Vec::new()),
            state: RwLock::new(ContextState::Created),
            operation: Mutex::new(()),
            resources,
            diagnostics: Mutex::new(diagnostics),
            close_started: AtomicBool::new(false),
            close_result,
        })
    }

    /// 返回串行化 refresh、start 与 close 的操作锁。
    pub(crate) const fn operation(&self) -> &Mutex<()> {
        &self.operation
    }

    /// 返回 Context 独占的内建运行资源。
    pub(crate) const fn resources(&self) -> &ContextResources {
        &self.resources
    }

    /// 把已经解析、需要在失败或关闭时释放的组件压入共享栈。
    pub(crate) async fn push_component(&self, component: Arc<dyn Lifecycle>) {
        self.components.lock().await.push(component);
    }

    /// 克隆当前组件栈，使启动过程不跨用户 Future 持有组件锁。
    pub(crate) async fn components(&self) -> Vec<Arc<dyn Lifecycle>> {
        self.components.lock().await.clone()
    }

    /// 取得组件栈所有权并清空共享位置，保证每个组件最多被一个回滚流程停止。
    pub(crate) async fn take_components(&self) -> Vec<Arc<dyn Lifecycle>> {
        mem::take(&mut *self.components.lock().await)
    }

    /// 返回当前生命周期状态快照。
    pub(crate) async fn state(&self) -> ContextState {
        *self.state.read().await
    }

    /// 原子替换可观察状态，并同步更新脱敏诊断快照。
    pub(crate) async fn set_state(&self, state: ContextState) {
        *self.state.write().await = state;
        self.diagnostics
            .lock()
            .await
            .set_context_state(state.as_str().to_owned());
    }

    /// 返回调用时刻拥有自身数据的诊断报告快照。
    ///
    /// `unused_definitions` 来自 Container 的同步解析追踪快照；这里只把它与同一
    /// 时刻的生命周期诊断值组合，不在协调器中保存第二份组件使用状态。
    pub(crate) async fn startup_report(&self, unused_definitions: Vec<String>) -> StartupReport {
        let mut report = self.diagnostics.lock().await.clone();
        report.set_unused_definitions(unused_definitions);
        report
    }

    /// 记录一个低基数静态运行期告警代码。
    pub(crate) async fn record_warning(&self, warning: &'static str) {
        self.diagnostics.lock().await.record_warning(warning);
    }

    /// 将生命周期超时转换成低基数、无业务正文的运行期告警代码。
    pub(crate) async fn record_lifecycle_warning(&self, error: &ContextError) {
        if let ContextError::LifecycleTimeout { abort_settled, .. } = error {
            self.record_warning("context.lifecycle-hook.timeout").await;
            if !abort_settled {
                self.record_warning("context.lifecycle-hook.abort-unsettled")
                    .await;
            }
        }
    }

    /// 追加一条不包含底层错误正文的生命周期观测。
    pub(crate) async fn record_observation(
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

    /// 停止应用任务监督器，并仅向公开诊断写入稳定错误代码。
    pub(crate) async fn shutdown_managed_tasks(&self) -> Option<ManagedTaskError> {
        let supervisor = self.resources.managed_tasks()?;
        let error = supervisor
            .shutdown(*self.resources.task_shutdown_policy())
            .await
            .err();
        if error.is_some() {
            self.record_warning("context.managed-task.shutdown-failed")
                .await;
        }
        error
    }

    /// 在独立 Tokio task 中执行一个组件停止钩子。
    ///
    /// 单个用户钩子 panic 只会转化为该组件的结构化失败，协调器仍会继续停止其余
    /// 组件。组件名在提交任务前取得，诊断路径不会读取用户错误正文。
    pub(crate) async fn stop_component(
        &self,
        component: &Arc<dyn Lifecycle>,
    ) -> Result<(), ContextError> {
        let started = Instant::now();
        let name = component.name();
        let owned_component = Arc::clone(component);
        let task = tokio::spawn(async move { owned_component.stop().await });
        let policy = *self.resources.lifecycle_execution_policy();
        let result = LifecycleTaskExecutor::execute(
            task,
            name,
            LifecyclePhase::Stop,
            policy.stop_timeout(),
            policy.abort_timeout(),
        )
        .await;
        let outcome = if result.is_ok() {
            DiagnosticOutcome::Succeeded
        } else {
            DiagnosticOutcome::Failed
        };
        self.record_observation(name, DiagnosticPhase::Stop, outcome, started)
            .await;
        if let Err(error) = &result {
            self.record_lifecycle_warning(error).await;
        }
        result
    }

    /// 逆序停止全部组件、隔离每个钩子的 panic，并保留第一个错误。
    pub(crate) async fn stop_all(&self, components: &[Arc<dyn Lifecycle>]) -> Option<ContextError> {
        let mut first_error = None;
        for component in components.iter().rev() {
            let stop_error = self.stop_component(component).await.err();
            if first_error.is_none() {
                first_error = stop_error;
            }
        }
        first_error
    }

    /// 启动唯一关闭协调任务并等待共享结果。
    ///
    /// 当前等待者被取消不会影响后台协调器。低层 Context 没有预绑定 Runtime 时，
    /// 会使用调用现场的 Tokio Handle；两者都不存在时返回结构化协调错误。
    pub(crate) async fn close(self: &Arc<Self>) -> CloseResult {
        self.start_close()?;
        self.wait_for_close().await
    }

    /// 原子提交唯一关闭任务，并用第二个轻量观察任务隔离协调器自身的 panic。
    fn start_close(self: &Arc<Self>) -> CloseResult {
        if self.close_started.load(Ordering::Acquire) {
            return Ok(());
        }

        let handle = self
            .resources
            .runtime()
            .cloned()
            .or_else(|| Handle::try_current().ok())
            .ok_or_else(|| ContextError::LifecycleCoordinator {
                operation: "close",
                source: Arc::new(std::io::Error::other(
                    "Tokio runtime is unavailable for context close",
                )),
            })?;
        if self
            .close_started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }

        // 先广播取消，再排队等待操作锁；这样并发运行的 start/refresh 能尽快观察到
        // 应用已经进入停机方向，同时关闭协调器仍负责最终的资源释放。
        self.resources.cancellation().cancel();
        let coordinator = Arc::clone(self);
        let close_task = handle.spawn(async move { coordinator.finish_close().await });
        let observer = Arc::clone(self);
        handle.spawn(async move {
            let result = match close_task.await {
                Ok(result) => result,
                Err(source) => {
                    observer
                        .record_warning("context.lifecycle-coordinator.failed")
                        .await;
                    observer.set_state(ContextState::Closed).await;
                    Err(ContextError::LifecycleCoordinator {
                        operation: "close",
                        source: Arc::new(source),
                    })
                }
            };
            observer.close_result.send_replace(Some(result));
        });
        Ok(())
    }

    /// 在后台完成任务排空、组件逆序停止和 Closed 终态发布。
    async fn finish_close(&self) -> CloseResult {
        let _operation = self.operation.lock().await;
        if self.state().await == ContextState::Closed {
            return Ok(());
        }

        let draining_state = if self.state().await == ContextState::Ready {
            ContextState::Draining
        } else {
            ContextState::RollingBack
        };
        self.set_state(draining_state).await;

        // 长期任务可能正在使用生命周期组件提供的连接池、消费者或调度器。因此先
        // 取消并等待任务退出，再取得组件栈所有权并按依赖逆序执行 stop。
        let task_error = self.shutdown_managed_tasks().await;
        let components = self.take_components().await;
        let lifecycle_error = self.stop_all(&components).await;
        self.set_state(ContextState::Closed).await;
        task_error
            .map(|source| ContextError::ManagedTask { source })
            .or(lifecycle_error)
            .map_or(Ok(()), Err)
    }

    /// 等待协调器发布可克隆结果；并发调用者不会重新执行关闭流程。
    async fn wait_for_close(&self) -> CloseResult {
        let mut receiver = self.close_result.subscribe();
        loop {
            if let Some(result) = receiver.borrow().clone() {
                return result;
            }
            if let Err(source) = receiver.changed().await {
                return Err(ContextError::LifecycleCoordinator {
                    operation: "close",
                    source: Arc::new(source),
                });
            }
        }
    }
}
