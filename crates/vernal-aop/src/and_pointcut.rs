//! 逻辑与组合切点对象。

use crate::{Operation, Pointcut};

/// 仅在左右两个切点都匹配时成立的组合切点。
///
/// 左右切点保持具体 Rust 类型，组合阶段不额外分配 trait object。整个表达式最终
/// 由 [`crate::Advisor`] 擦除并只在应用计划构建阶段求值。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AndPointcut<L, R> {
    left: L,
    right: R,
}

impl<L, R> AndPointcut<L, R> {
    /// 创建逻辑与组合。
    #[must_use]
    pub const fn new(left: L, right: R) -> Self {
        Self { left, right }
    }

    /// 返回左侧切点。
    #[must_use]
    pub const fn left(&self) -> &L {
        &self.left
    }

    /// 返回右侧切点。
    #[must_use]
    pub const fn right(&self) -> &R {
        &self.right
    }
}

impl<L, R> Pointcut for AndPointcut<L, R>
where
    L: Pointcut,
    R: Pointcut,
{
    /// 短路求值：左侧不匹配时不再访问右侧。
    fn matches(&self, operation: &Operation) -> bool {
        self.left.matches(operation) && self.right.matches(operation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn and_pointcut_matches_both() {
        let pc = AndPointcut::new(
            |op: &Operation| op.component() == "Service",
            |op: &Operation| op.method() == "method",
        );
        let op = Operation::new("Service", "method");
        assert!(pc.matches(&op));
    }

    #[test]
    fn and_pointcut_fails_first() {
        let pc = AndPointcut::new(
            |op: &Operation| op.component() == "Other",
            |op: &Operation| op.method() == "method",
        );
        let op = Operation::new("Service", "method");
        assert!(!pc.matches(&op));
    }

    #[test]
    fn and_pointcut_fails_second() {
        let pc = AndPointcut::new(
            |op: &Operation| op.component() == "Service",
            |op: &Operation| op.method() == "other",
        );
        let op = Operation::new("Service", "method");
        assert!(!pc.matches(&op));
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn and_pointcut_left() {
        let pc = AndPointcut::new(
            |op: &Operation| op.component() == "Service",
            |op: &Operation| op.method() == "method",
        );
        let op = Operation::new("Service", "method");
        assert!(pc.left()(&op));
    }

    #[test]
    fn and_pointcut_right() {
        let pc = AndPointcut::new(
            |op: &Operation| op.component() == "Service",
            |op: &Operation| op.method() == "method",
        );
        let op = Operation::new("Service", "method");
        assert!(pc.right()(&op));
    }

    #[test]
    fn and_pointcut_clone() {
        let pc = AndPointcut::new(
            |op: &Operation| op.component() == "Service",
            |op: &Operation| op.method() == "method",
        );
        let cloned = pc.clone();
        let op = Operation::new("Service", "method");
        assert!(cloned.matches(&op));
    }

    #[test]
    fn and_pointcut_debug() {
        let pc = AndPointcut::new(true, true);
        let debug = format!("{:?}", pc);
        assert!(!debug.is_empty());
    }

    #[test]
    fn and_pointcut_partial_eq() {
        let pc1 = AndPointcut::new(true, true);
        let pc2 = AndPointcut::new(true, true);
        assert_eq!(pc1, pc2);
    }

    #[test]
    fn and_pointcut_not_equal() {
        let pc1 = AndPointcut::new(true, true);
        let pc2 = AndPointcut::new(true, false);
        assert_ne!(pc1, pc2);
    }
}
