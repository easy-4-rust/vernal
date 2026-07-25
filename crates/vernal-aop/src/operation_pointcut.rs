//! 精确操作切点对象。

use crate::{Operation, Pointcut};

/// 只匹配一个完整 [`Operation`] 身份的切点。
///
/// 组件名和方法名必须同时相等。该对象适合权限、事务或幂等能力只应用于一个
/// 明确业务操作的场景，比在每个调用方重复手写比较闭包更容易审计。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationPointcut {
    operation: Operation,
}

impl OperationPointcut {
    /// 使用目标操作创建精确切点。
    #[must_use]
    pub const fn new(operation: Operation) -> Self {
        Self { operation }
    }

    /// 返回被匹配的目标操作。
    #[must_use]
    pub const fn operation(&self) -> &Operation {
        &self.operation
    }
}

impl Pointcut for OperationPointcut {
    /// 同时比较组件名与方法名。
    fn matches(&self, operation: &Operation) -> bool {
        operation == &self.operation
    }
}
