//! 始终匹配的切点。
//!
//! 对应 spring-aop `org.springframework.aop.TruePointcut`。

use crate::Operation;
use crate::pointcut::Pointcut;

/// 始终匹配的切点单例。
///
/// 对应 spring-aop `TruePointcut`。
///
/// 这是一个单例，始终返回 `true`。
// 对标 Spring AOP 的 API 脚手架：TruePointcut 为始终匹配的切点单例，暂未被内部调用。
#[allow(dead_code)]
pub struct TruePointcut;

impl Pointcut for TruePointcut {
    fn matches(&self, _operation: &Operation) -> bool {
        true
    }
}

impl std::fmt::Debug for TruePointcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TruePointcut")
    }
}

impl std::fmt::Display for TruePointcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pointcut.TRUE")
    }
}

/// 获取 TruePointcut 单例。
// 对标 Spring AOP 的 API 脚手架：true_pointcut 返回 TruePointcut 单例引用，暂未被内部调用。
#[allow(dead_code)]
pub fn true_pointcut() -> &'static TruePointcut {
    &TruePointcut
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_pointcut_matches_all() {
        let pointcut = TruePointcut;
        let op = Operation::new("Service", "method");
        assert!(pointcut.matches(&op));
    }

    #[test]
    fn true_pointcut_display() {
        let pointcut = TruePointcut;
        assert_eq!(format!("{}", pointcut), "Pointcut.TRUE");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn true_pointcut_matches_all() {
        let pointcut = TruePointcut;
        let op1 = Operation::new("Service1", "method1");
        let op2 = Operation::new("Service2", "method2");
        assert!(pointcut.matches(&op1));
        assert!(pointcut.matches(&op2));
    }

    #[test]
    fn true_pointcut_display() {
        let pointcut = TruePointcut;
        assert_eq!(format!("{}", pointcut), "Pointcut.TRUE");
    }

    #[test]
    fn true_pointcut_debug() {
        let pointcut = TruePointcut;
        let debug = format!("{:?}", pointcut);
        assert!(!debug.is_empty());
    }

    #[test]
    fn true_pointcut_singleton() {
        let p1 = true_pointcut();
        let p2 = true_pointcut();
        // Both point to the same static reference
        assert!(std::ptr::eq(p1, p2));
    }
}
