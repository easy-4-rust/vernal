//! task 支持包。
//!
//! 对标 Spring `org.springframework.core.task.support` 包：任务装饰器。

mod composite_task_decorator;
mod context_propagating_task_decorator;

pub use composite_task_decorator::CompositeTaskDecorator;
pub use context_propagating_task_decorator::ContextPropagatingTaskDecorator;
