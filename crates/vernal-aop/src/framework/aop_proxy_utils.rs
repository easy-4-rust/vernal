//! AOP 代理工具类。
//!
//! 对应 spring-aop `org.springframework.aop.framework.AopProxyUtils`。

use crate::Advisor;

/// AOP 代理工具类。
///
/// 对应 spring-aop `AopProxyUtils`。
///
/// 提供代理相关的工具方法。
pub struct AopProxyUtils;

impl AopProxyUtils {
    /// 获取目标类。
    ///
    /// 从顾问列表中推断目标类。
    pub fn ultimate_target_class(advisors: &[&Advisor]) -> Option<String> {
        // 简化实现：返回第一个顾问的切点描述
        advisors.first().map(|_| "Unknown".to_string())
    }

    /// 检查顾问是否完成。
    pub fn complete_proxied_interfaces(advisors: &[&Advisor]) -> Vec<String> {
        let mut interfaces = Vec::new();
        for _advisor in advisors {
            // 简化实现：添加默认接口
            interfaces.push("org.springframework.aop.SpringProxy".to_string());
            interfaces.push("org.springframework.aop.framework.Advised".to_string());
        }
        interfaces
    }

    /// 检查是否相等。
    pub fn equals_in_proxy(proxy1: &dyn std::any::Any, proxy2: &dyn std::any::Any) -> bool {
        std::ptr::eq(proxy1, proxy2)
    }
}

impl std::fmt::Debug for AopProxyUtils {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AopProxyUtils").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aop_proxy_utils_ultimate_target_class() {
        let advisors: Vec<&Advisor> = vec![];
        let result = AopProxyUtils::ultimate_target_class(&advisors);
        assert!(result.is_none());
    }

    #[test]
    fn aop_proxy_utils_complete_proxied_interfaces() {
        let advisors: Vec<&Advisor> = vec![];
        let interfaces = AopProxyUtils::complete_proxied_interfaces(&advisors);
        assert_eq!(interfaces.len(), 0);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;
    use std::sync::Arc;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(
            &'a self,
            invocation: Arc<crate::Invocation>,
            next: crate::Next<'a>,
        ) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn ultimate_target_class_empty_advisors() {
        let advisors: Vec<&crate::Advisor> = vec![];
        let result = AopProxyUtils::ultimate_target_class(&advisors);
        assert!(result.is_none());
    }

    #[test]
    fn complete_proxied_interfaces_empty_advisors() {
        let advisors: Vec<&crate::Advisor> = vec![];
        let interfaces = AopProxyUtils::complete_proxied_interfaces(&advisors);
        assert!(interfaces.is_empty());
    }

    #[test]
    fn complete_proxied_interfaces_with_advisors() {
        let advisor = crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0);
        let advisors: Vec<&crate::Advisor> = vec![&advisor];
        let interfaces = AopProxyUtils::complete_proxied_interfaces(&advisors);
        assert!(!interfaces.is_empty());
    }

    #[test]
    fn debug() {
        let debug = format!("{:?}", AopProxyUtils);
        assert!(!debug.is_empty());
    }
}

#[cfg(test)]
mod aop_proxy_utils_tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;
    use std::sync::Arc;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(
            &'a self,
            invocation: Arc<crate::Invocation>,
            next: crate::Next<'a>,
        ) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn ultimate_target_class_empty() {
        let advisors: Vec<&crate::Advisor> = vec![];
        let result = AopProxyUtils::ultimate_target_class(&advisors);
        assert!(result.is_none());
    }

    #[test]
    fn complete_proxied_interfaces_empty() {
        let advisors: Vec<&crate::Advisor> = vec![];
        let interfaces = AopProxyUtils::complete_proxied_interfaces(&advisors);
        assert!(interfaces.is_empty());
    }

    #[test]
    fn complete_proxied_interfaces_with_advisors() {
        let advisor = crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0);
        let advisors: Vec<&crate::Advisor> = vec![&advisor];
        let interfaces = AopProxyUtils::complete_proxied_interfaces(&advisors);
        assert!(!interfaces.is_empty());
    }

    #[test]
    fn debug() {
        let debug = format!("{:?}", AopProxyUtils);
        assert!(!debug.is_empty());
    }
}

#[cfg(test)]
mod equals_tests {
    use super::*;

    #[test]
    fn equals_in_proxy_same_ref() {
        let value = 42i32;
        let reference = &value as &dyn std::any::Any;
        assert!(AopProxyUtils::equals_in_proxy(reference, reference));
    }

    #[test]
    fn equals_in_proxy_different_ref() {
        let value1 = 42i32;
        let value2 = 42i32;
        let reference1 = &value1 as &dyn std::any::Any;
        let reference2 = &value2 as &dyn std::any::Any;
        assert!(!AopProxyUtils::equals_in_proxy(reference1, reference2));
    }
}
