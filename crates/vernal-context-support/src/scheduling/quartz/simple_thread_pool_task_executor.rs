//! 简单线程池任务执行器 — 对标 `SimpleThreadPoolTaskExecutor`。

use std::collections::VecDeque;
use std::sync::Mutex;

/// 简单线程池任务执行器。
pub struct SimpleThreadPoolTaskExecutor {
    thread_count: usize,
    queue: Mutex<VecDeque<Box<dyn FnOnce() + Send>>>,
    wait_for_jobs: bool,
}
impl SimpleThreadPoolTaskExecutor {
    /// 创建简单线程池任务执行器。
    pub fn new(thread_count: usize) -> Self {
        Self {
            thread_count,
            queue: Mutex::new(VecDeque::new()),
            wait_for_jobs: false,
        }
    }
    /// 设置关闭时是否等待任务完成。
    pub fn set_wait_for_jobs_to_complete_on_shutdown(&mut self, wait: bool) {
        self.wait_for_jobs = wait;
    }
    /// 执行任务（入队等待处理）。
    pub fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        if let Ok(mut q) = self.queue.lock() {
            q.push_back(task);
        }
    }
    /// 获取线程数量。
    pub fn thread_count(&self) -> usize {
        self.thread_count
    }
    /// 关闭线程池。
    pub fn shutdown(&self, _wait: bool) {}
    /// 是否等待任务完成后再关闭。
    pub fn is_wait_for_jobs(&self) -> bool {
        self.wait_for_jobs
    }
}
