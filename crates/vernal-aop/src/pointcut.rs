//! 切点匹配契约。

use crate::Operation;

/// Rust 原生 Pointcut DSL 子模块。
pub mod dsl;

/// 判断一个切面是否应用于指定操作。
///
/// 闭包会自动实现该 trait，因此调用方既可以定义可复用切点对象，也可以直接
/// 使用 `|operation: &Operation| ...` 完成精确、前缀或标签式匹配。
pub trait Pointcut: Send + Sync + 'static {
    /// 返回当前切点是否匹配操作。
    fn matches(&self, operation: &Operation) -> bool;
}

impl<F> Pointcut for F
where
    F: Fn(&Operation) -> bool + Send + Sync + 'static,
{
    fn matches(&self, operation: &Operation) -> bool {
        self(operation)
    }
}
