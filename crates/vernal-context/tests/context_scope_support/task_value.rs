//! 应用上下文测试用任务级组件对象。

/// 验证高层 Context 可以解析自定义作用域组件。
pub struct TaskValue {
    /// 用于确认工厂实际执行的值。
    pub value: &'static str,
}
