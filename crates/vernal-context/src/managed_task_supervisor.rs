//! 应用级 Tokio 后台任务监督器对象。

use std::{
    error::Error,
    future::Future,
    sync::{Arc, Mutex},
};

use tokio::{
    runtime::Handle,
    sync::{Notify, watch},
    task::JoinError,
    time::timeout,
};
use tokio_util::sync::CancellationToken;
use vernal_core::SharedError;

use crate::{
    ManagedTaskError, ManagedTaskId, TaskShutdownPolicy, managed_task_registry::ManagedTaskRegistry,
};

type TaskResult = Result<(), SharedError>;
type ShutdownResult = Result<(), ManagedTaskError>;

/// 持有应用后台 Tokio 任务并统一传播失败、取消与停机结果。
///
/// 业务组件把长期 Worker、配置监听、消息消费或 Hutool-Rust Cron 驱动任务提交给
/// 本对象，而不是直接遗弃 `JoinHandle`。任一任务返回错误、panic 或在应用取消前
/// 被终止时，监督器保存第一个结构化失败、停止接受新任务并取消应用令牌。
///
/// 停机采用唯一后台协调任务；调用者取消 `shutdown()` Future 不会遗弃清理。
/// 多个等待者共享同一结果，关闭过程不会重复 abort 或重复消费任务错误。
pub struct ManagedTaskSupervisor {
    runtime: Arc<Handle>,
    cancellation: Arc<CancellationToken>,
    registry: Mutex<ManagedTaskRegistry>,
    idle: Notify,
    shutdown_result: watch::Sender<Option<ShutdownResult>>,
}

impl ManagedTaskSupervisor {
    /// 使用应用拥有的 Tokio Runtime 与取消令牌创建共享监督器。
    #[must_use]
    pub fn new(runtime: Arc<Handle>, cancellation: Arc<CancellationToken>) -> Arc<Self> {
        let (shutdown_result, _) = watch::channel(None);
        Arc::new(Self {
            runtime,
            cancellation,
            registry: Mutex::new(ManagedTaskRegistry::default()),
            idle: Notify::new(),
            shutdown_result,
        })
    }

    /// 提交一个需要跟随应用生命周期的 Tokio 后台任务。
    ///
    /// 任务名称要求为静态字符串，防止请求参数、租户标识或凭证进入诊断维度。
    /// Future 应监听 [`Self::cancellation_token`]；若在优雅等待上限内没有退出，
    /// 停机协调器会通过 Tokio `AbortHandle` 强制终止。
    ///
    /// # Errors
    ///
    /// 监督器已经观察到关键任务失败或开始停机时返回
    /// [`ManagedTaskError::SpawnRejected`]。
    pub fn spawn<Fut, E>(
        self: &Arc<Self>,
        name: &'static str,
        future: Fut,
    ) -> Result<ManagedTaskId, ManagedTaskError>
    where
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        E: Error + Send + Sync + 'static,
    {
        if name.trim().is_empty() {
            return Err(ManagedTaskError::InvalidName);
        }

        // 用户 Future 已由调用方创建；临界区内只向 Tokio 提交任务和登记
        // AbortHandle，不执行任何用户代码或异步等待。
        let (id, task) = {
            let mut registry = self
                .registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !registry.accepting {
                return Err(ManagedTaskError::SpawnRejected { task: name });
            }
            let id = ManagedTaskId::new(registry.next_id);
            registry.next_id = registry
                .next_id
                .checked_add(1)
                .ok_or(ManagedTaskError::IdentifierExhausted { task: name })?;
            let task = self.runtime.spawn(async move {
                future
                    .await
                    .map_err(|source| Arc::new(source) as SharedError)
            });
            registry.tasks.insert(id, (name, task.abort_handle()));
            (id, task)
        };

        // 观察器独立等待用户任务，把返回错误、panic 和异常取消统一恢复成
        // Context 可处理的结构化结果。JoinHandle 始终被消费，不会静默 detach。
        let supervisor = Arc::clone(self);
        self.runtime.spawn(async move {
            supervisor.complete(id, name, task.await);
        });
        Ok(id)
    }

    /// 派生一个 `AsyncTask` 组件到 Tokio runtime。
    ///
    /// 对标 tx_di 的 `App::comp_run`：在 `ApplicationContext::start()` 后
    /// 自动派生所有实现 `AsyncTask` 的组件。
    ///
    /// # 参数
    /// - `task`：实现 `AsyncTask` 的组件
    ///
    /// # Errors
    ///
    /// 监督器已经观察到关键任务失败或开始停机时返回
    /// [`ManagedTaskError::SpawnRejected`]。
    pub fn spawn_async_task<T: crate::AsyncTask>(
        self: &Arc<Self>,
        task: Arc<T>,
    ) -> Result<ManagedTaskId, ManagedTaskError> {
        let name = task.name();
        let token = self.cancellation_token();
        // 将 LifecycleFuture 转换为 spawn 可接受的类型
        let adapted = async move {
            task.run(token).await.map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })
        };
        self.spawn(name, adapted)
    }

    /// 带超时的任务派生。
    ///
    /// 与 [`Self::spawn`] 相同，但为任务附加单个超时限制。
    /// 超时后任务被 abort，等效于调用方自行用 `tokio::time::timeout` 包裹。
    ///
    /// # 参数
    /// - `name`：任务静态名称（用于诊断）
    /// - `task_timeout`：单个任务超时
    /// - `future`：要监督的异步任务
    ///
    /// # Errors
    ///
    /// 监督器已经观察到关键任务失败或开始停机时返回
    /// [`ManagedTaskError::SpawnRejected`]。
    pub fn spawn_with_timeout<Fut>(
        self: &Arc<Self>,
        name: &'static str,
        task_timeout: std::time::Duration,
        future: Fut,
    ) -> Result<ManagedTaskId, ManagedTaskError>
    where
        Fut: Future<Output = Result<(), std::io::Error>> + Send + 'static,
    {
        // 用 tokio::time::timeout 包裹原始 Future
        let timed_future = async move {
            tokio::time::timeout(task_timeout, future)
                .await
                // 超时返回 Err(Elapsed)，转换为 io::Error::TimedOut
                .unwrap_or(Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "任务超时",
                )))
        };
        // 委托给标准 spawn，复用观察器和完成逻辑
        self.spawn(name, timed_future)
    }

    /// 返回供单个任务监听的应用取消树子令牌。
    ///
    /// 调用方通常在提交任务前取得该令牌，并在循环中与业务输入一起 `select!`。
    /// 取消子令牌不会反向取消应用；应用关闭或关键任务失败仍会向下传播。
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.child_token()
    }

    /// 返回当前尚未由观察器收口的任务数量。
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .tasks
            .len()
    }

    /// 返回监督器已经收口的任务总数。
    #[must_use]
    pub fn completed_count(&self) -> u64 {
        self.registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .completed_count
    }

    /// 返回监督器是否仍接受新任务。
    #[must_use]
    pub fn is_accepting(&self) -> bool {
        self.registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .accepting
    }

    /// 返回运行期间观察到的第一个任务失败。
    ///
    /// 返回值可克隆，适合 Context 在 refresh/start 边界执行健康检查；原始错误
    /// 仍只通过显式错误链暴露。
    #[must_use]
    pub fn failure(&self) -> Option<ManagedTaskError> {
        self.registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first_failure
            .clone()
    }

    /// 启动取消安全的两阶段停机并等待共享结果。
    ///
    /// 第一个调用者冻结任务注册并提交唯一协调任务；其他调用者只订阅结果。
    /// 丢弃任一等待 Future 不会取消停机协调器。
    ///
    /// # Errors
    ///
    /// 返回运行期间的第一个任务失败，或优雅/强制终止阶段的结构化超时。
    pub async fn shutdown(
        self: &Arc<Self>,
        policy: TaskShutdownPolicy,
    ) -> Result<(), ManagedTaskError> {
        self.start_shutdown(policy);
        self.wait_for_shutdown().await
    }

    /// 把一个用户任务的 Tokio Join 结果写入监督器状态。
    fn complete(
        &self,
        id: ManagedTaskId,
        name: &'static str,
        result: Result<TaskResult, JoinError>,
    ) {
        let failure = match result {
            Ok(Err(source)) => Some(ManagedTaskError::TaskFailed { task: name, source }),
            Err(source) if source.is_panic() => Some(ManagedTaskError::TaskPanicked {
                task: name,
                source: Arc::new(source),
            }),
            Err(source) if !self.cancellation.is_cancelled() => {
                Some(ManagedTaskError::TaskCancelled {
                    task: name,
                    source: Arc::new(source),
                })
            }
            Ok(Ok(())) | Err(_) => None,
        };

        let should_cancel = {
            let mut registry = self
                .registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            registry.tasks.remove(&id);
            registry.completed_count = registry.completed_count.saturating_add(1);
            if registry.first_failure.is_none() {
                if let Some(failure) = failure {
                    registry.first_failure = Some(failure);
                    registry.accepting = false;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        if should_cancel {
            self.cancellation.cancel();
        }
        self.idle.notify_waiters();
    }

    /// 原子提交唯一停机协调任务。
    fn start_shutdown(self: &Arc<Self>, policy: TaskShutdownPolicy) {
        let should_start = {
            let mut registry = self
                .registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if registry.shutdown_started {
                false
            } else {
                registry.accepting = false;
                registry.shutdown_started = true;
                true
            }
        };
        if !should_start {
            return;
        }

        self.cancellation.cancel();
        let supervisor = Arc::clone(self);
        self.runtime.spawn(async move {
            let result = supervisor.finish_shutdown(policy).await;
            supervisor.shutdown_result.send_replace(Some(result));
        });
    }

    /// 执行优雅等待、超时 abort 和最终失败选择。
    async fn finish_shutdown(&self, policy: TaskShutdownPolicy) -> ShutdownResult {
        if timeout(policy.graceful_timeout(), self.wait_until_idle())
            .await
            .is_err()
        {
            let remaining = self.active_count();
            self.abort_all();
            if timeout(policy.abort_timeout(), self.wait_until_idle())
                .await
                .is_err()
            {
                return Err(ManagedTaskError::AbortTimeout {
                    timeout: policy.abort_timeout(),
                    remaining: self.active_count(),
                });
            }
            return Err(ManagedTaskError::ShutdownTimeout {
                timeout: policy.graceful_timeout(),
                remaining,
            });
        }

        self.registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first_failure
            .clone()
            .map_or(Ok(()), Err)
    }

    /// 等待活动任务表为空，并避免检查与订阅之间丢失最后一次唤醒。
    async fn wait_until_idle(&self) {
        loop {
            let notified = self.idle.notified();
            if self.active_count() == 0 {
                return;
            }
            notified.await;
        }
    }

    /// 向当前仍存活的所有用户任务发送 Tokio abort。
    fn abort_all(&self) {
        let handles = self
            .registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .tasks
            .values()
            .map(|(_, handle)| handle.clone())
            .collect::<Vec<_>>();
        for handle in handles {
            handle.abort();
        }
    }

    /// 等待停机协调器发布可克隆结果。
    async fn wait_for_shutdown(&self) -> ShutdownResult {
        let mut receiver = self.shutdown_result.subscribe();
        loop {
            if let Some(result) = receiver.borrow().clone() {
                return result;
            }
            if receiver.changed().await.is_err() {
                return Err(ManagedTaskError::CoordinatorUnavailable {
                    remaining: self.active_count(),
                });
            }
        }
    }
}
