//! 任务装饰器契约。
//!
//! 对标 Spring `org.springframework.core.task.TaskDecorator`。

/// 任务装饰器契约。
///
/// 对应 Java: org.springframework.core.task.TaskDecorator
///
/// Spring 语义：在任务提交给执行器前包装任务（如注入上下文传播）。
pub trait TaskDecorator: Send + Sync {
    /// 装饰任务。
    ///
    /// 对应 Java: `TaskDecorator#decorate(Runnable)`
    fn decorate(
        &self,
        task: Box<dyn FnOnce() + Send + 'static>,
    ) -> Box<dyn FnOnce() + Send + 'static>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingDecorator {
        count: Arc<AtomicUsize>,
    }

    impl TaskDecorator for CountingDecorator {
        fn decorate(
            &self,
            task: Box<dyn FnOnce() + Send + 'static>,
        ) -> Box<dyn FnOnce() + Send + 'static> {
            let count = self.count.clone();
            Box::new(move || {
                count.fetch_add(1, Ordering::SeqCst);
                task();
            })
        }
    }

    #[test]
    fn decorator_wraps_task() {
        // A 类（合同对齐）：对标 Spring 任务包装
        let count = Arc::new(AtomicUsize::new(0));
        let decorator = CountingDecorator {
            count: count.clone(),
        };
        let task = Box::new(|| {}) as Box<dyn FnOnce() + Send + 'static>;
        let wrapped = decorator.decorate(task);
        wrapped();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
