//! 简单线程池任务执行器 — 对标 `SimpleThreadPoolTaskExecutor`。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// 简单线程池任务执行器。
pub struct SimpleThreadPoolTaskExecutor {
    thread_count: usize,
    queue: Mutex<VecDeque<Box<dyn FnOnce() + Send>>>,
    wait_for_jobs: bool,
}
impl SimpleThreadPoolTaskExecutor {
    pub fn new(thread_count: usize) -> Self {
        Self {
            thread_count,
            queue: Mutex::new(VecDeque::new()),
            wait_for_jobs: false,
        }
    }
    pub fn set_wait_for_jobs_to_complete_on_shutdown(&mut self, wait: bool) {
        self.wait_for_jobs = wait;
    }
    pub fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        if let Ok(mut q) = self.queue.lock() {
            q.push_back(task);
        }
    }
    pub fn thread_count(&self) -> usize {
        self.thread_count
    }
    pub fn shutdown(&self, _wait: bool) {}
    pub fn is_wait_for_jobs(&self) -> bool {
        self.wait_for_jobs
    }
}
