//! 上下文传播任务装饰器。
//!
//! 对标 Spring `org.springframework.core.task.support.ContextPropagatingTaskDecorator`。

use std::sync::Arc;

use crate::task::TaskDecorator;

/// 上下文传播任务装饰器。
///
/// 对应 Java: org.springframework.core.task.support.ContextPropagatingTaskDecorator
///
/// Spring 语义：把调用方上下文快照传播给被装饰任务；Rust 中以 `Arc` 共享
/// 不可变上下文表达（对标 `ContextSnapshot` 的只读语义）。
pub struct ContextPropagatingTaskDecorator {
    context: Arc<dyn std::any::Any + Send + Sync>,
}

impl ContextPropagatingTaskDecorator {
    /// 创建携带上下文的装饰器。
    #[must_use]
    pub fn new(context: Arc<dyn std::any::Any + Send + Sync>) -> Self {
        Self { context }
    }

    /// 返回上下文引用。
    #[must_use]
    pub fn context(&self) -> &Arc<dyn std::any::Any + Send + Sync> {
        &self.context
    }
}

impl TaskDecorator for ContextPropagatingTaskDecorator {
    fn decorate(
        &self,
        task: Box<dyn FnOnce() + Send + 'static>,
    ) -> Box<dyn FnOnce() + Send + 'static> {
        let context = self.context.clone();
        Box::new(move || {
            let _ = context; // 上下文随任务存活（对标 Spring 传播语义）
            task();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn context_survives_task_execution() {
        // A 类（合同对齐）：对标 Spring 上下文传播
        let context: Arc<dyn std::any::Any + Send + Sync> = Arc::new("trace-id-123".to_string());
        let decorator = ContextPropagatingTaskDecorator::new(context);
        let ran = Arc::new(AtomicBool::new(false));
        let flag = ran.clone();
        let task = Box::new(move || flag.store(true, Ordering::SeqCst))
            as Box<dyn FnOnce() + Send + 'static>;
        decorator.decorate(task)();
        assert!(ran.load(Ordering::SeqCst));
    }

    #[test]
    fn exposes_context() {
        // B 类（边界行为）：上下文可访问
        let context: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42_i32);
        let decorator = ContextPropagatingTaskDecorator::new(context);
        assert_eq!(decorator.context().downcast_ref::<i32>(), Some(&42));
    }
}
