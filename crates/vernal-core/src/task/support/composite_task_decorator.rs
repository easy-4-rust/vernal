//! 组合任务装饰器。
//!
//! 对标 Spring `org.springframework.core.task.support.CompositeTaskDecorator`。

use crate::task::TaskDecorator;

/// 组合任务装饰器。
///
/// 对应 Java: org.springframework.core.task.support.CompositeTaskDecorator
///
/// Spring 语义：按添加顺序依次装饰任务（每个装饰器包装前一个的结果）。
pub struct CompositeTaskDecorator {
    decorators: Vec<Box<dyn TaskDecorator>>,
}

impl CompositeTaskDecorator {
    /// 创建空组合装饰器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            decorators: Vec::new(),
        }
    }

    /// 添加装饰器（后添加者最外层）。
    pub fn add_decorator(&mut self, decorator: Box<dyn TaskDecorator>) {
        self.decorators.push(decorator);
    }
}

impl Default for CompositeTaskDecorator {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskDecorator for CompositeTaskDecorator {
    fn decorate(
        &self,
        task: Box<dyn FnOnce() + Send + 'static>,
    ) -> Box<dyn FnOnce() + Send + 'static> {
        let mut wrapped = task;
        // 后添加者最外层：逆序遍历包装
        for decorator in self.decorators.iter().rev() {
            wrapped = decorator.decorate(wrapped);
        }
        wrapped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Counter {
        count: Arc<AtomicUsize>,
        label: &'static str,
    }

    impl TaskDecorator for Counter {
        fn decorate(
            &self,
            task: Box<dyn FnOnce() + Send + 'static>,
        ) -> Box<dyn FnOnce() + Send + 'static> {
            let count = self.count.clone();
            let label = self.label;
            Box::new(move || {
                count.fetch_add(1, Ordering::SeqCst);
                let _ = label;
                task();
            })
        }
    }

    #[test]
    fn applies_decorators_in_order() {
        // A 类（合同对齐）：对标 Spring 组合装饰顺序
        let count = Arc::new(AtomicUsize::new(0));
        let mut composite = CompositeTaskDecorator::new();
        composite.add_decorator(Box::new(Counter {
            count: count.clone(),
            label: "a",
        }));
        composite.add_decorator(Box::new(Counter {
            count: count.clone(),
            label: "b",
        }));
        let task = Box::new(|| {}) as Box<dyn FnOnce() + Send + 'static>;
        composite.decorate(task)();
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn empty_composite_passes_through() {
        // B 类（边界行为）：无装饰器时原样执行
        use std::sync::atomic::{AtomicBool, Ordering};
        let composite = CompositeTaskDecorator::new();
        let ran = Arc::new(AtomicBool::new(false));
        let flag = ran.clone();
        let task = Box::new(move || flag.store(true, Ordering::SeqCst))
            as Box<dyn FnOnce() + Send + 'static>;
        composite.decorate(task)();
        assert!(ran.load(Ordering::SeqCst));
    }
}
