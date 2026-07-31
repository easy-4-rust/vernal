//! 拦截器和动态方法匹配器。
//!
//! 对应 spring-aop `org.springframework.aop.framework.InterceptorAndDynamicMethodMatcher`。

use std::sync::Arc;

use crate::Interceptor;
use crate::Operation;
use crate::method_matcher::MethodMatcher;

/// 拦截器和动态方法匹配器。
///
/// 对应 spring-aop `InterceptorAndDynamicMethodMatcher`。
///
/// 内部使用类，将拦截器与动态方法匹配器配对。
/// 用于在运行时检查方法是否匹配。
pub struct InterceptorAndDynamicMethodMatcher {
    /// 拦截器。
    interceptor: Arc<dyn Interceptor>,
    /// 动态方法匹配器。
    method_matcher: Arc<dyn MethodMatcher>,
}

impl InterceptorAndDynamicMethodMatcher {
    /// 创建新的拦截器和动态方法匹配器。
    pub fn new(interceptor: Arc<dyn Interceptor>, method_matcher: Arc<dyn MethodMatcher>) -> Self {
        Self {
            interceptor,
            method_matcher,
        }
    }

    /// 获取拦截器。
    pub fn interceptor(&self) -> &Arc<dyn Interceptor> {
        &self.interceptor
    }

    /// 获取方法匹配器。
    pub fn method_matcher(&self) -> &Arc<dyn MethodMatcher> {
        &self.method_matcher
    }

    /// 检查方法是否匹配。
    pub fn matches(&self, operation: &Operation) -> bool {
        self.method_matcher.matches(operation)
    }

    /// 是否为动态匹配器。
    pub fn is_runtime(&self) -> bool {
        self.method_matcher.is_runtime()
    }
}

impl std::fmt::Debug for InterceptorAndDynamicMethodMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InterceptorAndDynamicMethodMatcher")
            .field("is_runtime", &self.method_matcher.is_runtime())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::true_method_matcher::TrueMethodMatcher;
    use crate::simple_interceptor::SimpleInterceptor;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: Arc<crate::Invocation>, next: crate::Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn interceptor_and_dynamic_method_matcher_creation() {
        let interceptor = Arc::new(TestInterceptor);
        let method_matcher = Arc::new(TrueMethodMatcher);

        let pair = InterceptorAndDynamicMethodMatcher::new(interceptor, method_matcher);
        assert!(!pair.is_runtime());
    }

    #[test]
    fn interceptor_and_dynamic_method_matcher_matches() {
        let interceptor = Arc::new(TestInterceptor);
        let method_matcher = Arc::new(TrueMethodMatcher);

        let pair = InterceptorAndDynamicMethodMatcher::new(interceptor, method_matcher);
        let op = Operation::new("Service", "method");
        assert!(pair.matches(&op));
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::Operation;
    use crate::true_method_matcher::TrueMethodMatcher;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: Arc<crate::Invocation>, next: crate::Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn interceptor_and_dynamic_method_matcher_debug() {
        let interceptor = Arc::new(TestInterceptor);
        let method_matcher = Arc::new(TrueMethodMatcher);
        let pair = InterceptorAndDynamicMethodMatcher::new(interceptor, method_matcher);
        let debug = format!("{:?}", pair);
        assert!(!debug.is_empty());
    }

    #[test]
    fn interceptor_and_dynamic_method_matcher_get_interceptor() {
        let interceptor = Arc::new(TestInterceptor);
        let method_matcher = Arc::new(TrueMethodMatcher);
        let pair = InterceptorAndDynamicMethodMatcher::new(interceptor.clone(), method_matcher);
        let _ = pair.interceptor();
    }

    #[test]
    fn interceptor_and_dynamic_method_matcher_get_method_matcher() {
        let interceptor = Arc::new(TestInterceptor);
        let method_matcher = Arc::new(TrueMethodMatcher);
        let pair = InterceptorAndDynamicMethodMatcher::new(interceptor, method_matcher.clone());
        let _ = pair.method_matcher();
    }
}
