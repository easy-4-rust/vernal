//! 测试用作用域计数组件对象。

/// 记录工厂调用序号，用于验证缓存和 Scope 隔离。
pub struct ScopedCounter {
    /// 当前实例对应的全局构造序号。
    pub sequence: usize,
}
