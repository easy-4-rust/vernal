//! 逻辑或组合切点对象。

use crate::{Operation, Pointcut};

/// 左右任一切点匹配即可成立的组合切点。
///
/// 该对象可以把多组明确操作合并为同一横切能力范围，同时继续复用不可变调用
/// 计划和稳定 Advisor 顺序。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrPointcut<L, R> {
    left: L,
    right: R,
}

impl<L, R> OrPointcut<L, R> {
    /// 创建逻辑或组合。
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

impl<L, R> Pointcut for OrPointcut<L, R>
where
    L: Pointcut,
    R: Pointcut,
{
    /// 短路求值：左侧已匹配时不再访问右侧。
    fn matches(&self, operation: &Operation) -> bool {
        self.left.matches(operation) || self.right.matches(operation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn or_pointcut_matches_first() {
        let pc = OrPointcut::new(
            |op: &Operation| op.component() == "Service",
            |op: &Operation| op.component() == "Other",
        );
        let op = Operation::new("Service", "method");
        assert!(pc.matches(&op));
    }

    #[test]
    fn or_pointcut_matches_second() {
        let pc = OrPointcut::new(
            |op: &Operation| op.component() == "Other",
            |op: &Operation| op.component() == "Service",
        );
        let op = Operation::new("Service", "method");
        assert!(pc.matches(&op));
    }

    #[test]
    fn or_pointcut_fails_both() {
        let pc = OrPointcut::new(
            |op: &Operation| op.component() == "Other",
            |op: &Operation| op.component() == "Another",
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
    fn or_pointcut_left() {
        let pc = OrPointcut::new(
            |op: &Operation| op.component() == "Service",
            |op: &Operation| op.component() == "Other",
        );
        let op = Operation::new("Service", "method");
        assert!(pc.left()(&op));
    }

    #[test]
    fn or_pointcut_right() {
        let pc = OrPointcut::new(
            |op: &Operation| op.component() == "Other",
            |op: &Operation| op.component() == "Service",
        );
        let op = Operation::new("Service", "method");
        assert!(pc.right()(&op));
    }

    #[test]
    fn or_pointcut_clone() {
        let pc = OrPointcut::new(
            |op: &Operation| op.component() == "Service",
            |op: &Operation| op.component() == "Other",
        );
        let cloned = pc.clone();
        let op = Operation::new("Service", "method");
        assert!(cloned.matches(&op));
    }

    #[test]
    fn or_pointcut_debug() {
        let pc = OrPointcut::new(true, false);
        let debug = format!("{:?}", pc);
        assert!(!debug.is_empty());
    }

    #[test]
    fn or_pointcut_partial_eq() {
        let pc1 = OrPointcut::new(true, false);
        let pc2 = OrPointcut::new(true, false);
        assert_eq!(pc1, pc2);
    }

    #[test]
    fn or_pointcut_not_equal() {
        let pc1 = OrPointcut::new(true, false);
        let pc2 = OrPointcut::new(false, true);
        assert_ne!(pc1, pc2);
    }
}
