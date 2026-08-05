//! 方法匹配器。
//!
//! 对应 spring-aop `MethodMatcher`。
//! 检查目标方法是否适合应用通知。

use std::fmt;

use crate::Operation;

/// 方法匹配器，判断方法是否匹配切点。
///
/// 对应 spring-aop `MethodMatcher`。
///
/// # 静态与动态匹配
///
/// - **静态匹配**：仅基于方法签名，结果可缓存
/// - **动态匹配**：需要运行时参数，每次调用都需重新评估
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::MethodMatcher;
///
/// // 匹配所有方法
/// let matcher = MethodMatcher::TRUE;
///
/// // 自定义匹配器
/// let matcher = MethodMatcher::from_fn(|op| op.method().starts_with("get"));
/// ```
pub trait MethodMatcher: Send + Sync + 'static {
    /// 静态匹配：基于方法签名判断是否匹配。
    fn matches(&self, operation: &Operation) -> bool;

    /// 是否为动态匹配器。
    ///
    /// 如果返回 `true`，则需要在运行时调用 `matches_runtime` 进行最终判断。
    fn is_runtime(&self) -> bool {
        false
    }

    /// 动态匹配：运行时基于参数判断是否匹配。
    ///
    /// 仅在 `is_runtime()` 返回 `true` 且静态匹配成功时调用。
    fn matches_runtime(&self, operation: &Operation, args: &[&dyn std::any::Any]) -> bool {
        let _ = args;
        self.matches(operation)
    }
}

/// 始终匹配的方法匹配器。
///
/// 对应 spring-aop `TrueMethodMatcher`。
#[derive(Clone, Copy, Debug)]
pub struct TrueMethodMatcher;

impl MethodMatcher for TrueMethodMatcher {
    fn matches(&self, _operation: &Operation) -> bool {
        true
    }

    fn is_runtime(&self) -> bool {
        false
    }
}

impl fmt::Display for TrueMethodMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MethodMatcher.TRUE")
    }
}

/// 基于闭包的静态方法匹配器。
#[derive(Clone)]
pub struct StaticMethodMatcher<F: Fn(&Operation) -> bool + Send + Sync + 'static> {
    predicate: F,
}

impl<F: Fn(&Operation) -> bool + Send + Sync + 'static> MethodMatcher for StaticMethodMatcher<F> {
    fn matches(&self, operation: &Operation) -> bool {
        (self.predicate)(operation)
    }

    fn is_runtime(&self) -> bool {
        false
    }
}

/// 动态方法匹配器。
///
/// 需要在运行时基于参数进行匹配。
#[derive(Clone)]
pub struct DynamicMethodMatcher<F>
where
    F: Fn(&Operation, &[&dyn std::any::Any]) -> bool + Send + Sync + 'static,
{
    static_predicate: fn(&Operation) -> bool,
    dynamic_predicate: F,
}

impl<F> MethodMatcher for DynamicMethodMatcher<F>
where
    F: Fn(&Operation, &[&dyn std::any::Any]) -> bool + Send + Sync + 'static,
{
    fn matches(&self, operation: &Operation) -> bool {
        (self.static_predicate)(operation)
    }

    fn is_runtime(&self) -> bool {
        true
    }

    fn matches_runtime(&self, operation: &Operation, args: &[&dyn std::any::Any]) -> bool {
        (self.dynamic_predicate)(operation, args)
    }
}

/// 方法匹配器工厂。
pub struct MethodMatcherFactory;

impl MethodMatcherFactory {
    /// 始终匹配的实例。
    pub const TRUE: TrueMethodMatcher = TrueMethodMatcher;

    /// 从闭包创建静态方法匹配器。
    pub fn from_fn<F: Fn(&Operation) -> bool + Send + Sync + 'static>(
        predicate: F,
    ) -> StaticMethodMatcher<F> {
        StaticMethodMatcher { predicate }
    }

    /// 创建动态方法匹配器。
    pub fn dynamic<F>(
        static_predicate: fn(&Operation) -> bool,
        dynamic_predicate: F,
    ) -> DynamicMethodMatcher<F>
    where
        F: Fn(&Operation, &[&dyn std::any::Any]) -> bool + Send + Sync + 'static,
    {
        DynamicMethodMatcher {
            static_predicate,
            dynamic_predicate,
        }
    }
}

/// 闭包自动实现 MethodMatcher（静态匹配）。
impl<F> MethodMatcher for F
where
    F: Fn(&Operation) -> bool + Send + Sync + 'static,
{
    fn matches(&self, operation: &Operation) -> bool {
        self(operation)
    }

    fn is_runtime(&self) -> bool {
        false
    }
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
    fn static_method_matcher() {
        let matcher =
            MethodMatcherFactory::from_fn(|op: &Operation| op.method().starts_with("get"));
        let get_op = Operation::new("Service", "getUser");
        let set_op = Operation::new("Service", "setUser");

        assert!(matcher.matches(&get_op));
        assert!(!matcher.matches(&set_op));
        assert!(!matcher.is_runtime());
    }

    #[test]
    fn dynamic_method_matcher() {
        let matcher = MethodMatcherFactory::dynamic(
            |_op| true, // 静态匹配所有
            |_op, args| {
                // 动态匹配：检查第一个参数是否为 "admin"
                args.first()
                    .and_then(|a| (*a).downcast_ref::<String>())
                    .map(|s| s == "admin")
                    .unwrap_or(false)
            },
        );

        let op = Operation::new("Service", "delete");
        assert!(matcher.matches(&op));
        assert!(matcher.is_runtime());

        let admin = String::from("admin");
        let user = String::from("user");
        assert!(matcher.matches_runtime(&op, &[&admin]));
        assert!(!matcher.matches_runtime(&op, &[&user]));
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn true_method_matcher_display() {
        let matcher = TrueMethodMatcher;
        assert_eq!(format!("{}", matcher), "MethodMatcher.TRUE");
    }

    #[test]
    fn static_method_matcher_is_runtime() {
        let matcher =
            MethodMatcherFactory::from_fn(|op: &Operation| op.method().starts_with("get"));
        assert!(!matcher.is_runtime());
    }

    #[test]
    fn static_method_matcher_matches() {
        let matcher =
            MethodMatcherFactory::from_fn(|op: &Operation| op.method().starts_with("get"));
        let op = Operation::new("Service", "getUser");
        assert!(matcher.matches(&op));
    }

    #[test]
    fn static_method_matcher_no_match() {
        let matcher =
            MethodMatcherFactory::from_fn(|op: &Operation| op.method().starts_with("get"));
        let op = Operation::new("Service", "setUser");
        assert!(!matcher.matches(&op));
    }

    #[test]
    fn dynamic_method_matcher_is_runtime() {
        let matcher = MethodMatcherFactory::dynamic(|_op| true, |_op, _args| true);
        assert!(matcher.is_runtime());
    }

    #[test]
    fn dynamic_method_matcher_matches_static() {
        let matcher =
            MethodMatcherFactory::dynamic(|op| op.method().starts_with("get"), |_op, _args| true);
        let op = Operation::new("Service", "getUser");
        assert!(matcher.matches(&op));
    }

    #[test]
    fn dynamic_method_matcher_no_match_static() {
        let matcher =
            MethodMatcherFactory::dynamic(|op| op.method().starts_with("get"), |_op, _args| true);
        let op = Operation::new("Service", "setUser");
        assert!(!matcher.matches(&op));
    }

    #[test]
    fn dynamic_method_matcher_matches_runtime() {
        let matcher = MethodMatcherFactory::dynamic(
            |_op| true,
            |_op, args| {
                args.first()
                    .and_then(|a| (*a).downcast_ref::<String>())
                    .map(|s| s == "admin")
                    .unwrap_or(false)
            },
        );
        let op = Operation::new("Service", "method");
        let admin = String::from("admin");
        assert!(matcher.matches_runtime(&op, &[&admin]));
    }

    #[test]
    fn dynamic_method_matcher_no_match_runtime() {
        let matcher = MethodMatcherFactory::dynamic(
            |_op| true,
            |_op, args| {
                args.first()
                    .and_then(|a| (*a).downcast_ref::<String>())
                    .map(|s| s == "admin")
                    .unwrap_or(false)
            },
        );
        let op = Operation::new("Service", "method");
        let user = String::from("user");
        assert!(!matcher.matches_runtime(&op, &[&user]));
    }
}
