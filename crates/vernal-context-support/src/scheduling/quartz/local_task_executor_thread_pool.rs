//! 本地任务执行器线程池 — 对标 `LocalTaskExecutorThreadPool`。

use std::sync::Arc;

/// 任务执行器 trait。
pub trait TaskExecutor: Send + Sync {
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>);
}

/// 本地任务执行器线程池。
pub struct LocalTaskExecutorThreadPool {
    task_executor: Option<Arc<dyn TaskExecutor>>,
}
impl LocalTaskExecutorThreadPool {
    pub fn new() -> Self {
        Self {
            task_executor: None,
        }
    }
    pub fn set_task_executor(&mut self, executor: Arc<dyn TaskExecutor>) {
        self.task_executor = Some(executor);
    }
    pub fn run_in_thread(&self, runnable: Box<dyn FnOnce() + Send + 'static>) -> bool {
        if let Some(executor) = &self.task_executor {
            executor.execute(runnable);
            true
        } else {
            false
        }
    }
    pub fn pool_size(&self) -> i32 {
        -1
    }
    pub fn block_for_available_threads(&self) -> i32 {
        1
    }
    pub fn shutdown(&self, _wait_for_jobs: bool) {}
}
impl Default for LocalTaskExecutorThreadPool {
    fn default() -> Self {
        Self::new()
    }
}
