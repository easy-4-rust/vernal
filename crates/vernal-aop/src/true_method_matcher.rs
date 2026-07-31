//! 始终匹配的方法匹配器。
//!
//! 对应 spring-aop `org.springframework.aop.TrueMethodMatcher`。

use crate::method_matcher::MethodMatcher;
use crate::Operation;

/// 始终匹配的方法匹配器单例。
///
/// 对应 spring-aop `TrueMethodMatcher`。
///
/// 这是一个单例，始终返回 `true`。
// 对标 Spring AOP 的 API 脚手架：TrueMethodMatcher 为 TruePointcut 的方法匹配单例，暂未被内部调用。
#[allow(dead_code)]
pub struct TrueMethodMatcher;

impl MethodMatcher for TrueMethodMatcher {
    fn matches(&self, _operation: &Operation) -> bool {
        true
    }

    fn is_runtime(&self) -> bool {
        false
    }
}

impl std::fmt::Debug for TrueMethodMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TrueMethodMatcher")
    }
}

impl std::fmt::Display for TrueMethodMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MethodMatcher.TRUE")
    }
}

/// 获取 TrueMethodMatcher 单例。
// 对标 Spring AOP 的 API 脚手架：true_method_matcher 返回 TrueMethodMatcher 单例引用，暂未被内部调用。
#[allow(dead_code)]
pub fn true_method_matcher() -> &'static TrueMethodMatcher {
    &TrueMethodMatcher
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_method_matcher_matches_all() {
        let matcher = TrueMethodMatcher;
        let op = Operation::new("Service", "method");
        assert!(matcher.matches(&op));
        assert!(!matcher.is_runtime());
    }

    #[test]
    fn true_method_matcher_display() {
        let matcher = TrueMethodMatcher;
        assert_eq!(format!("{}", matcher), "MethodMatcher.TRUE");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn true_method_matcher_matches_various() {
        let matcher = TrueMethodMatcher;
        let op1 = Operation::new("Service1", "method1");
        let op2 = Operation::new("Service2", "method2");
        let op3 = Operation::new("Service3", "method3");
        assert!(matcher.matches(&op1));
        assert!(matcher.matches(&op2));
        assert!(matcher.matches(&op3));
    }

    #[test]
    fn true_method_matcher_is_runtime_false() {
        let matcher = TrueMethodMatcher;
        assert!(!matcher.is_runtime());
    }

    #[test]
    fn true_method_matcher_display() {
        let matcher = TrueMethodMatcher;
        assert_eq!(format!("{}", matcher), "MethodMatcher.TRUE");
    }

    #[test]
    fn true_method_matcher_debug() {
        let matcher = TrueMethodMatcher;
        let debug = format!("{:?}", matcher);
        assert!(!debug.is_empty());
    }

    #[test]
    fn true_method_matcher_singleton() {
        let m1 = true_method_matcher();
        let m2 = true_method_matcher();
        assert!(std::ptr::eq(m1, m2));
    }
}
