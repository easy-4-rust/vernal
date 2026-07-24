//! 受管任务与生命周期关闭顺序测试对象。

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use vernal_context::{Lifecycle, LifecycleFuture, ManagedTaskSupervisor};

/// 在 start 阶段提交长期任务，并在 stop 阶段验证任务已经退出。
pub struct TaskOwningLifecycle {
    supervisor: Arc<ManagedTaskSupervisor>,
    task_completed: Arc<AtomicBool>,
    stop_observed_completion: AtomicBool,
}

impl TaskOwningLifecycle {
    /// 使用应用注入的任务监督器创建测试组件。
    pub fn new(supervisor: Arc<ManagedTaskSupervisor>) -> Self {
        Self {
            supervisor,
            task_completed: Arc::new(AtomicBool::new(false)),
            stop_observed_completion: AtomicBool::new(false),
        }
    }

    /// 返回长期任务是否已经响应应用取消并退出。
    pub fn task_completed(&self) -> bool {
        self.task_completed.load(Ordering::SeqCst)
    }

    /// 返回 stop 钩子是否观察到任务已经退出。
    pub fn stop_observed_completion(&self) -> bool {
        self.stop_observed_completion.load(Ordering::SeqCst)
    }
}

impl Lifecycle for TaskOwningLifecycle {
    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        let cancellation = self.supervisor.cancellation_token();
        let task_completed = Arc::clone(&self.task_completed);
        let result = self.supervisor.spawn("test.lifecycle-worker", async move {
            cancellation.cancelled().await;
            task_completed.store(true, Ordering::SeqCst);
            Ok::<_, io::Error>(())
        });
        Box::pin(async move {
            result
                .map(|_| ())
                .map_err(|source| Box::new(source) as vernal_core::BoxError)
        })
    }

    fn stop(&self) -> LifecycleFuture<'_> {
        self.stop_observed_completion
            .store(self.task_completed(), Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}
