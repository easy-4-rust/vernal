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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn operation_pointcut_matches() {
        let op = Operation::new("Service", "method");
        let pc = OperationPointcut::new(op.clone());
        assert!(pc.matches(&op));
    }

    #[test]
    fn operation_pointcut_no_match() {
        let op1 = Operation::new("Service", "method1");
        let op2 = Operation::new("Service", "method2");
        let pc = OperationPointcut::new(op1);
        assert!(!pc.matches(&op2));
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn operation_pointcut_new() {
        let op = Operation::new("Service", "method");
        let pc = OperationPointcut::new(op.clone());
        assert_eq!(pc.operation().component(), "Service");
        assert_eq!(pc.operation().method(), "method");
    }

    #[test]
    fn operation_pointcut_matches_same() {
        let op = Operation::new("Service", "method");
        let pc = OperationPointcut::new(op.clone());
        assert!(pc.matches(&op));
    }

    #[test]
    fn operation_pointcut_no_match_different_component() {
        let op1 = Operation::new("Service1", "method");
        let op2 = Operation::new("Service2", "method");
        let pc = OperationPointcut::new(op1);
        assert!(!pc.matches(&op2));
    }

    #[test]
    fn operation_pointcut_no_match_different_method() {
        let op1 = Operation::new("Service", "method1");
        let op2 = Operation::new("Service", "method2");
        let pc = OperationPointcut::new(op1);
        assert!(!pc.matches(&op2));
    }

    #[test]
    fn operation_pointcut_clone() {
        let op = Operation::new("Service", "method");
        let pc = OperationPointcut::new(op.clone());
        let cloned = pc.clone();
        assert_eq!(cloned.operation().component(), "Service");
    }

    #[test]
    fn operation_pointcut_debug() {
        let op = Operation::new("Service", "method");
        let pc = OperationPointcut::new(op);
        let debug = format!("{:?}", pc);
        assert!(!debug.is_empty());
    }

    #[test]
    fn operation_pointcut_partial_eq() {
        let op = Operation::new("Service", "method");
        let pc1 = OperationPointcut::new(op.clone());
        let pc2 = OperationPointcut::new(op);
        assert_eq!(pc1, pc2);
    }

    #[test]
    fn operation_pointcut_not_equal() {
        let op1 = Operation::new("Service", "method1");
        let op2 = Operation::new("Service", "method2");
        let pc1 = OperationPointcut::new(op1);
        let pc2 = OperationPointcut::new(op2);
        assert_ne!(pc1, pc2);
    }
}
