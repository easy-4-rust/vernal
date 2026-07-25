//! `IoC` 管理的周期任务执行计划对象。

use std::{future::Future, pin::Pin, sync::Arc};

use tokio::time::{Instant, MissedTickBehavior};
use tokio_util::sync::CancellationToken;
use vernal_core::SharedError;
use vernal_ioc::{ComponentKey, Container, Qualifier, ResolveError};

use crate::{
    ContextError, ManagedTaskSupervisor, ScheduledTask, ScheduledTaskFailure, TaskSchedule,
    TaskScheduleMode,
};

type ScheduledTaskFuture =
    Pin<Box<dyn Future<Output = Result<(), ScheduledTaskFailure>> + Send + 'static>>;
type ScheduledTaskStarter = dyn Fn(&Container, Arc<ManagedTaskSupervisor>) -> Result<(), ContextError>
    + Send
    + Sync
    + 'static;

/// 保存一个具体周期任务组件的解析与受管启动计划。
///
/// 类型擦除只存在于 Context 的启动计划中。具体任务仍从最终 Container 解析，
/// 并以同一个 `Arc<T>` 在应用生命周期内重复执行，不进行全局查找或名称 downcast。
pub(crate) struct ManagedScheduledTask {
    component: ComponentKey,
    starter: Arc<ScheduledTaskStarter>,
}

impl ManagedScheduledTask {
    /// 声明一个无限定符 Singleton 周期任务。
    #[must_use]
    pub(crate) fn new<T>() -> Self
    where
        T: ScheduledTask,
    {
        Self::with_resolver::<T, _>(ComponentKey::of::<T>(), |container| {
            container.resolve::<T>()
        })
    }

    /// 声明一个带精确限定符的 Singleton 周期任务。
    #[must_use]
    pub(crate) fn qualified<T>(qualifier: Qualifier) -> Self
    where
        T: ScheduledTask,
    {
        let component = ComponentKey::qualified::<T>(qualifier.clone());
        Self::with_resolver::<T, _>(component, move |container| {
            container.resolve_qualified::<T>(&qualifier)
        })
    }

    /// 创建绑定具体组件解析方式的类型擦除启动计划。
    fn with_resolver<T, F>(component: ComponentKey, resolver: F) -> Self
    where
        T: ScheduledTask,
        F: Fn(&Container) -> Result<Arc<T>, ResolveError> + Send + Sync + 'static,
    {
        let error_component = component.clone();
        let starter: Arc<ScheduledTaskStarter> = Arc::new(move |container, managed_tasks| {
            let task =
                resolver(container).map_err(|source| ContextError::ScheduledTaskResolution {
                    component: error_component.clone(),
                    source: Box::new(source),
                })?;
            let schedule = task.schedule();
            let name = task.name();
            let cancellation = managed_tasks.cancellation_token();
            let future = Self::execute(task, schedule, cancellation, name);
            managed_tasks
                .spawn(name, future)
                .map(|_| ())
                .map_err(|source| ContextError::ManagedTask { source })
        });
        Self { component, starter }
    }

    /// 驱动一个具体任务，直到应用取消或单次执行失败。
    fn execute<T>(
        task: Arc<T>,
        schedule: TaskSchedule,
        cancellation: CancellationToken,
        name: &'static str,
    ) -> ScheduledTaskFuture
    where
        T: ScheduledTask,
    {
        Box::pin(async move {
            match schedule.mode() {
                TaskScheduleMode::FixedDelay => {
                    Self::run_fixed_delay(task, schedule, cancellation, name).await
                }
                TaskScheduleMode::FixedRate => {
                    Self::run_fixed_rate(task, schedule, cancellation, name).await
                }
            }
        })
    }

    /// 以上一次完成时刻为基准，串行执行固定延迟任务。
    async fn run_fixed_delay<T>(
        task: Arc<T>,
        schedule: TaskSchedule,
        cancellation: CancellationToken,
        name: &'static str,
    ) -> Result<(), ScheduledTaskFailure>
    where
        T: ScheduledTask,
    {
        if Self::wait_or_cancel(schedule.initial_delay(), &cancellation).await {
            return Ok(());
        }
        loop {
            Self::run_once(Arc::clone(&task), cancellation.clone(), name).await?;
            if Self::wait_or_cancel(schedule.interval(), &cancellation).await {
                return Ok(());
            }
        }
    }

    /// 沿固定时间轴串行执行任务，并跳过执行期间错过的触发时刻。
    async fn run_fixed_rate<T>(
        task: Arc<T>,
        schedule: TaskSchedule,
        cancellation: CancellationToken,
        name: &'static str,
    ) -> Result<(), ScheduledTaskFailure>
    where
        T: ScheduledTask,
    {
        let first_tick = Instant::now() + schedule.initial_delay();
        let mut interval = tokio::time::interval_at(first_tick, schedule.interval());
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                biased;
                () = cancellation.cancelled() => return Ok(()),
                _ = interval.tick() => {
                    Self::run_once(Arc::clone(&task), cancellation.clone(), name).await?;
                }
            }
        }
    }

    /// 执行一次用户任务并把原始错误包进默认脱敏的失败对象。
    async fn run_once<T>(
        task: Arc<T>,
        cancellation: CancellationToken,
        name: &'static str,
    ) -> Result<(), ScheduledTaskFailure>
    where
        T: ScheduledTask,
    {
        task.run(cancellation)
            .await
            .map_err(|source| ScheduledTaskFailure::new(name, Arc::new(source) as SharedError))
    }

    /// 等待指定时长；应用取消时立即返回 `true`。
    async fn wait_or_cancel(delay: std::time::Duration, cancellation: &CancellationToken) -> bool {
        if delay.is_zero() {
            return cancellation.is_cancelled();
        }
        tokio::select! {
            biased;
            () = cancellation.cancelled() => true,
            () = tokio::time::sleep(delay) => false,
        }
    }

    /// 返回周期任务组件的稳定身份。
    #[must_use]
    pub(crate) const fn component(&self) -> &ComponentKey {
        &self.component
    }

    /// 返回违反应用级后台任务身份要求的组件作用域。
    pub(crate) fn invalid_scope(&self, container: &Container) -> Option<&'static str> {
        container
            .registry()
            .definitions()
            .iter()
            .find(|definition| definition.key() == &self.component)
            .map(|definition| definition.scope())
            .filter(|scope| !scope.is_singleton())
            .map(vernal_ioc::Scope::as_str)
    }

    /// 从最终 Container 解析任务并提交给统一监督器。
    pub(crate) fn start(
        &self,
        container: &Container,
        managed_tasks: Arc<ManagedTaskSupervisor>,
    ) -> Result<(), ContextError> {
        (self.starter)(container, managed_tasks)
    }
}
