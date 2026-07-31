//! 逻辑非组合切点对象。

use crate::{Operation, Pointcut};

/// 对一个切点匹配结果取反的组合切点。
///
/// 常用于“全局能力排除健康检查”一类规则。排除条件仍在计划预编译阶段完成，
/// 不会让热路径携带负向判断。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotPointcut<P> {
    inner: P,
}

impl<P> NotPointcut<P> {
    /// 创建逻辑非组合。
    #[must_use]
    pub const fn new(inner: P) -> Self {
        Self { inner }
    }

    /// 返回被取反的内部切点。
    #[must_use]
    pub const fn inner(&self) -> &P {
        &self.inner
    }
}

impl<P> Pointcut for NotPointcut<P>
where
    P: Pointcut,
{
    /// 返回内部匹配结果的逻辑反值。
    fn matches(&self, operation: &Operation) -> bool {
        !self.inner.matches(operation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn not_pointcut_inverts_match() {
        let pc = NotPointcut::new(|op: &Operation| op.component() == "Service");
        let op = Operation::new("Service", "method");
        assert!(!pc.matches(&op));
    }

    #[test]
    fn not_pointcut_inverts_no_match() {
        let pc = NotPointcut::new(|op: &Operation| op.component() == "Other");
        let op = Operation::new("Service", "method");
        assert!(pc.matches(&op));
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn not_pointcut_inner() {
        let pc = NotPointcut::new(|op: &Operation| op.component() == "Service");
        let op = Operation::new("Service", "method");
        assert!(pc.inner()(&op));
    }

    #[test]
    fn not_pointcut_clone() {
        let pc = NotPointcut::new(|_: &Operation| true);
        let cloned = pc.clone();
        let op = Operation::new("test", "test");
        assert!(!cloned.matches(&op));
    }
}
