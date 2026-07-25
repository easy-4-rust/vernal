//! 全操作切点对象。

use crate::{Operation, Pointcut};

/// 匹配应用中全部已声明操作的切点。
///
/// 该对象适合日志、指标、追踪等真正跨越全部组件的横切能力。它是零状态值，
/// 不读取运行期上下文；匹配只发生在调用计划预编译阶段，因此不会给业务调用热
/// 路径增加判断。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnyPointcut;

impl AnyPointcut {
    /// 创建全操作切点。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Pointcut for AnyPointcut {
    /// 对任意操作返回 `true`。
    fn matches(&self, _operation: &Operation) -> bool {
        true
    }
}
