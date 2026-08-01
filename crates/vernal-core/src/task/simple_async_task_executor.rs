//! 简单异步任务执行器。
//!
//! 对标 Spring `org.springframework.core.task.SimpleAsyncTaskExecutor`。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::task::TaskExecutor;

/// 简单异步任务执行器。
///
/// 对应 Java: org.springframework.core.task.SimpleAsyncTaskExecutor
///
/// Spring 语义：每次 `execute` 启动一个新线程执行任务（无池化），
/// `threadNamePrefix` 用于诊断命名（对标 Spring 默认 `SimpleAsyncTaskExecutor-`）。
pub struct SimpleAsyncTaskExecutor {
    thread_name_prefix: String,
    sequence: Arc<AtomicU64>,
}

impl SimpleAsyncTaskExecutor {
    /// 创建使用默认名称前缀的执行器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            thread_name_prefix: "SimpleAsyncTaskExecutor-".to_string(),
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 创建使用自定义名称前缀的执行器。
    #[must_use]
    pub fn with_name_prefix(prefix: impl Into<String>) -> Self {
        Self {
            thread_name_prefix: prefix.into(),
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 返回名称前缀。
    #[must_use]
    pub fn thread_name_prefix(&self) -> &str {
        &self.thread_name_prefix
    }
}

impl Default for SimpleAsyncTaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskExecutor for SimpleAsyncTaskExecutor {
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        let name = format!(
            "{}{}",
            self.thread_name_prefix,
            self.sequence.fetch_add(1, Ordering::Relaxed)
        );
        std::thread::Builder::new()
            .name(name)
            .spawn(task)
            .expect("无法启动异步任务线程");
    }

    fn name(&self) -> &'static str {
        "simpleAsync"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn executes_task_on_new_thread() {
        // A 类（合同对齐）：对标 Spring 异步执行
        let executor = SimpleAsyncTaskExecutor::new();
        let (tx, rx) = mpsc::channel();
        executor.execute(Box::new(move || {
            tx.send("done").unwrap();
        }));
        assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap(), "done");
    }

    #[test]
    fn custom_name_prefix_applies() {
        // B 类（边界行为）：对标 Spring threadNamePrefix
        let executor = SimpleAsyncTaskExecutor::with_name_prefix("worker-");
        assert_eq!(executor.thread_name_prefix(), "worker-");
    }

    #[test]
    fn default_name_prefix() {
        let executor = SimpleAsyncTaskExecutor::default();
        assert_eq!(executor.thread_name_prefix(), "SimpleAsyncTaskExecutor-");
    }
}
