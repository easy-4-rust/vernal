//! 同步任务执行器。
//!
//! 对标 Spring `org.springframework.core.task.SyncTaskExecutor`。

use crate::task::TaskExecutor;

/// 同步任务执行器。
///
/// 对应 Java: org.springframework.core.task.SyncTaskExecutor
///
/// Spring 语义：任务在调用线程内同步执行（`execute` 返回时任务已完成）。
pub struct SyncTaskExecutor;

impl TaskExecutor for SyncTaskExecutor {
    fn execute(&self, task: Box<dyn FnOnce() + Send + 'static>) {
        task();
    }

    fn name(&self) -> &'static str {
        "sync"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn executes_task_synchronously() {
        // A 类（合同对齐）：对标 Spring 同步执行语义
        let executed = Arc::new(AtomicBool::new(false));
        let executor = SyncTaskExecutor;
        let flag = executed.clone();
        executor.execute(Box::new(move || {
            flag.store(true, Ordering::SeqCst);
        }));
        assert!(executed.load(Ordering::SeqCst), "任务应在返回前完成");
    }

    #[test]
    fn exposes_name() {
        // B 类（边界行为）：诊断名称
        let executor = SyncTaskExecutor;
        assert_eq!(executor.name(), "sync");
    }
}
