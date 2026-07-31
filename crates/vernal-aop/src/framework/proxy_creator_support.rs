//! 代理创建器支持类。
//!
//! 对应 spring-aop `org.springframework.aop.framework.ProxyCreatorSupport`。

use std::sync::Arc;

use crate::Advisor;
use crate::Interceptor;
use crate::target_source::TargetSource;
use super::advised_support::AdvisedSupport;
use super::aop_proxy::AopProxy;

/// 代理创建器支持类。
///
/// 对应 spring-aop `ProxyCreatorSupport`。
///
/// 代理工厂的基类，提供代理创建的通用功能。
pub struct ProxyCreatorSupport {
    /// 通知支持类。
    advised: AdvisedSupport,
    /// 代理实例（缓存）。
    proxy: Option<Box<dyn AopProxy>>,
}

impl ProxyCreatorSupport {
    /// 创建新的代理创建器支持类。
    pub fn new() -> Self {
        Self {
            advised: AdvisedSupport::new(),
            proxy: None,
        }
    }

    /// 创建带目标源的代理创建器支持类。
    pub fn with_target_source(target_source: Arc<dyn TargetSource>) -> Self {
        Self {
            advised: AdvisedSupport::with_target_source(target_source),
            proxy: None,
        }
    }

    /// 获取通知支持类。
    pub fn advised(&self) -> &AdvisedSupport {
        &self.advised
    }

    /// 获取可变通知支持类。
    pub fn advised_mut(&mut self) -> &mut AdvisedSupport {
        &mut self.advised
    }

    /// 设置目标源。
    pub fn set_target_source(&mut self, target_source: Arc<dyn TargetSource>) {
        self.advised.set_target_source(target_source);
    }

    /// 添加顾问。
    pub fn add_advisor(&mut self, advisor: Arc<Advisor>) {
        self.advised.add_advisor(advisor);
        self.proxy = None; // 使缓存失效
    }

    /// 添加拦截器。
    pub fn add_interceptor(&mut self, interceptor: Arc<dyn Interceptor>) {
        let advisor = Arc::new(Advisor::shared(
            Arc::new(crate::any_pointcut::AnyPointcut::new()),
            interceptor,
            0,
        ));
        self.add_advisor(advisor);
    }

    /// 添加接口。
    pub fn add_interface(&mut self, interface: impl Into<String>) {
        self.advised.add_interface(interface);
    }

    /// 获取所有接口。
    pub fn get_interfaces(&self) -> &[String] {
        self.advised.get_interfaces()
    }

    /// 冻结配置。
    pub fn freeze(&mut self) {
        self.advised.freeze();
    }

    /// 是否冻结。
    pub fn is_frozen(&self) -> bool {
        self.advised.is_frozen()
    }

    /// 创建代理。
    ///
    /// 子类应覆盖此方法以提供实际的代理创建逻辑。
    pub fn create_proxy(&mut self) -> Result<&dyn AopProxy, Box<dyn std::error::Error + Send + Sync>> {
        if self.proxy.is_none() {
            // 子类应覆盖此方法
            return Err("create_proxy() not implemented - subclass must override".into());
        }
        Ok(self.proxy.as_ref().unwrap().as_ref())
    }

    /// 从另一个代理创建器复制配置。
    pub fn copy_from(&mut self, other: &ProxyCreatorSupport) {
        self.advised.copy_from(&other.advised);
        self.proxy = None;
    }
}

impl Default for ProxyCreatorSupport {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ProxyCreatorSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyCreatorSupport")
            .field("advised", &self.advised)
            .field("has_proxy", &self.proxy.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target_source::TargetSource;
    use crate::target_source_error::TargetSourceError;

    struct TestTargetSource;

    impl TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            Some("TestTarget")
        }

        fn get_target(&self) -> Result<Box<dyn std::any::Any>, TargetSourceError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn proxy_creator_support_creation() {
        let support = ProxyCreatorSupport::new();
        assert_eq!(support.advised().get_advisor_count(), 0);
        assert!(!support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let support = ProxyCreatorSupport::with_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_add_interface() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_freeze() {
        let mut support = ProxyCreatorSupport::new();
        support.freeze();
        assert!(support.is_frozen());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: Arc<crate::Invocation>, next: crate::Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    struct TestTargetSource;
    impl crate::target_source::TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            Some("TestTarget")
        }
        fn get_target(&self) -> Result<Box<dyn std::any::Any>, crate::target_source_error::TargetSourceError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn proxy_creator_support_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let support = ProxyCreatorSupport::with_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_set_target_source() {
        let mut support = ProxyCreatorSupport::new();
        let target_source = Arc::new(TestTargetSource);
        support.set_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_add_advisor() {
        let mut support = ProxyCreatorSupport::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        support.add_advisor(advisor);
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interceptor() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interceptor(Arc::new(TestInterceptor));
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interface() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_freeze() {
        let mut support = ProxyCreatorSupport::new();
        support.freeze();
        assert!(support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_copy_from() {
        let mut support1 = ProxyCreatorSupport::new();
        support1.add_interface("com.example.MyInterface");

        let mut support2 = ProxyCreatorSupport::new();
        support2.copy_from(&support1);
        assert_eq!(support2.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_debug() {
        let support = ProxyCreatorSupport::new();
        let debug = format!("{:?}", support);
        assert!(!debug.is_empty());
    }

    #[test]
    fn proxy_creator_support_default() {
        let support = ProxyCreatorSupport::default();
        assert_eq!(support.advised().get_advisor_count(), 0);
    }
}

#[cfg(test)]
mod proxy_creator_tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: Arc<crate::Invocation>, next: crate::Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    #[test]
    fn proxy_creator_support_advised_mut() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.advised_mut().get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_create_proxy_not_implemented() {
        let mut support = ProxyCreatorSupport::new();
        let result = support.create_proxy();
        assert!(result.is_err());
    }

    #[test]
    fn proxy_creator_support_multiple_advisors() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interceptor(Arc::new(TestInterceptor));
        support.add_interceptor(Arc::new(TestInterceptor));
        assert_eq!(support.advised().get_advisor_count(), 2);
    }

    #[test]
    fn proxy_creator_support_copy_from_preserves_advisors() {
        let mut support1 = ProxyCreatorSupport::new();
        support1.add_interceptor(Arc::new(TestInterceptor));

        let mut support2 = ProxyCreatorSupport::new();
        support2.copy_from(&support1);
        assert_eq!(support2.advised().get_advisor_count(), 1);
    }
}

#[cfg(test)]
mod proxy_creator_final_tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: Arc<crate::Invocation>, next: crate::Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    struct TestTargetSource;
    impl crate::target_source::TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            Some("TestTarget")
        }
        fn get_target(&self) -> Result<Box<dyn std::any::Any>, crate::target_source_error::TargetSourceError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn proxy_creator_support_new() {
        let support = ProxyCreatorSupport::new();
        assert_eq!(support.advised().get_advisor_count(), 0);
        assert!(!support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let support = ProxyCreatorSupport::with_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_set_target_source() {
        let mut support = ProxyCreatorSupport::new();
        let target_source = Arc::new(TestTargetSource);
        support.set_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_add_advisor() {
        let mut support = ProxyCreatorSupport::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        support.add_advisor(advisor);
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interface() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_freeze() {
        let mut support = ProxyCreatorSupport::new();
        support.freeze();
        assert!(support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_copy_from() {
        let mut support1 = ProxyCreatorSupport::new();
        support1.add_interface("com.example.MyInterface");

        let mut support2 = ProxyCreatorSupport::new();
        support2.copy_from(&support1);
        assert_eq!(support2.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_debug() {
        let support = ProxyCreatorSupport::new();
        let debug = format!("{:?}", support);
        assert!(!debug.is_empty());
    }

    #[test]
    fn proxy_creator_support_default() {
        let support = ProxyCreatorSupport::default();
        assert_eq!(support.advised().get_advisor_count(), 0);
    }
}

#[cfg(test)]
mod proxy_creator_coverage_tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: Arc<crate::Invocation>, next: crate::Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    struct TestTargetSource;
    impl crate::target_source::TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            Some("TestTarget")
        }
        fn get_target(&self) -> Result<Box<dyn std::any::Any>, crate::target_source_error::TargetSourceError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn proxy_creator_support_new_empty() {
        let support = ProxyCreatorSupport::new();
        assert_eq!(support.advised().get_advisor_count(), 0);
        assert!(!support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let support = ProxyCreatorSupport::with_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_set_target_source() {
        let mut support = ProxyCreatorSupport::new();
        let target_source = Arc::new(TestTargetSource);
        support.set_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_add_advisor() {
        let mut support = ProxyCreatorSupport::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        support.add_advisor(advisor);
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interceptor() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interceptor(Arc::new(TestInterceptor));
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interface() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_freeze() {
        let mut support = ProxyCreatorSupport::new();
        support.freeze();
        assert!(support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_copy_from() {
        let mut support1 = ProxyCreatorSupport::new();
        support1.add_interface("com.example.MyInterface");

        let mut support2 = ProxyCreatorSupport::new();
        support2.copy_from(&support1);
        assert_eq!(support2.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_debug() {
        let support = ProxyCreatorSupport::new();
        let debug = format!("{:?}", support);
        assert!(!debug.is_empty());
    }

    #[test]
    fn proxy_creator_support_default() {
        let support = ProxyCreatorSupport::default();
        assert_eq!(support.advised().get_advisor_count(), 0);
    }

    #[test]
    fn proxy_creator_support_advised() {
        let support = ProxyCreatorSupport::new();
        let _ = support.advised();
    }

    #[test]
    fn proxy_creator_support_advised_mut() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.advised_mut().get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_create_proxy_not_implemented() {
        let mut support = ProxyCreatorSupport::new();
        let result = support.create_proxy();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod proxy_creator_final_coverage_tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: Arc<crate::Invocation>, next: crate::Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    struct TestTargetSource;
    impl crate::target_source::TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            Some("TestTarget")
        }
        fn get_target(&self) -> Result<Box<dyn std::any::Any>, crate::target_source_error::TargetSourceError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn proxy_creator_support_new() {
        let support = ProxyCreatorSupport::new();
        assert_eq!(support.advised().get_advisor_count(), 0);
        assert!(!support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let support = ProxyCreatorSupport::with_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_set_target_source() {
        let mut support = ProxyCreatorSupport::new();
        let target_source = Arc::new(TestTargetSource);
        support.set_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_add_advisor() {
        let mut support = ProxyCreatorSupport::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        support.add_advisor(advisor);
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interceptor() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interceptor(Arc::new(TestInterceptor));
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interface() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_freeze() {
        let mut support = ProxyCreatorSupport::new();
        support.freeze();
        assert!(support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_copy_from() {
        let mut support1 = ProxyCreatorSupport::new();
        support1.add_interface("com.example.MyInterface");

        let mut support2 = ProxyCreatorSupport::new();
        support2.copy_from(&support1);
        assert_eq!(support2.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_debug() {
        let support = ProxyCreatorSupport::new();
        let debug = format!("{:?}", support);
        assert!(!debug.is_empty());
    }

    #[test]
    fn proxy_creator_support_default() {
        let support = ProxyCreatorSupport::default();
        assert_eq!(support.advised().get_advisor_count(), 0);
    }

    #[test]
    fn proxy_creator_support_advised() {
        let support = ProxyCreatorSupport::new();
        let _ = support.advised();
    }

    #[test]
    fn proxy_creator_support_advised_mut() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.advised_mut().get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_create_proxy_not_implemented() {
        let mut support = ProxyCreatorSupport::new();
        let result = support.create_proxy();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod proxy_creator_final_coverage {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: Arc<crate::Invocation>, next: crate::Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    struct TestTargetSource;
    impl crate::target_source::TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            Some("TestTarget")
        }
        fn get_target(&self) -> Result<Box<dyn std::any::Any>, crate::target_source_error::TargetSourceError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn proxy_creator_support_new() {
        let support = ProxyCreatorSupport::new();
        assert_eq!(support.advised().get_advisor_count(), 0);
        assert!(!support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let support = ProxyCreatorSupport::with_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_set_target_source() {
        let mut support = ProxyCreatorSupport::new();
        let target_source = Arc::new(TestTargetSource);
        support.set_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_add_advisor() {
        let mut support = ProxyCreatorSupport::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        support.add_advisor(advisor);
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interceptor() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interceptor(Arc::new(TestInterceptor));
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interface() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_freeze() {
        let mut support = ProxyCreatorSupport::new();
        support.freeze();
        assert!(support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_copy_from() {
        let mut support1 = ProxyCreatorSupport::new();
        support1.add_interface("com.example.MyInterface");

        let mut support2 = ProxyCreatorSupport::new();
        support2.copy_from(&support1);
        assert_eq!(support2.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_debug() {
        let support = ProxyCreatorSupport::new();
        let debug = format!("{:?}", support);
        assert!(!debug.is_empty());
    }

    #[test]
    fn proxy_creator_support_default() {
        let support = ProxyCreatorSupport::default();
        assert_eq!(support.advised().get_advisor_count(), 0);
    }

    #[test]
    fn proxy_creator_support_advised() {
        let support = ProxyCreatorSupport::new();
        let _ = support.advised();
    }

    #[test]
    fn proxy_creator_support_advised_mut() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.advised_mut().get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_create_proxy_not_implemented() {
        let mut support = ProxyCreatorSupport::new();
        let result = support.create_proxy();
        assert!(result.is_err());
    }
}


#[cfg(test)]
mod proxy_creator_last_coverage {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

    struct TestInterceptor;
    impl crate::Interceptor for TestInterceptor {
        fn intercept<'a>(&'a self, invocation: Arc<crate::Invocation>, next: crate::Next<'a>) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    struct TestTargetSource;
    impl crate::target_source::TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            Some("TestTarget")
        }
        fn get_target(&self) -> Result<Box<dyn std::any::Any>, crate::target_source_error::TargetSourceError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn proxy_creator_support_new() {
        let support = ProxyCreatorSupport::new();
        assert_eq!(support.advised().get_advisor_count(), 0);
        assert!(!support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let support = ProxyCreatorSupport::with_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_set_target_source() {
        let mut support = ProxyCreatorSupport::new();
        let target_source = Arc::new(TestTargetSource);
        support.set_target_source(target_source);
        assert!(support.advised().get_target_source().is_some());
    }

    #[test]
    fn proxy_creator_support_add_advisor() {
        let mut support = ProxyCreatorSupport::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        support.add_advisor(advisor);
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interceptor() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interceptor(Arc::new(TestInterceptor));
        assert_eq!(support.advised().get_advisor_count(), 1);
    }

    #[test]
    fn proxy_creator_support_add_interface() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_freeze() {
        let mut support = ProxyCreatorSupport::new();
        support.freeze();
        assert!(support.is_frozen());
    }

    #[test]
    fn proxy_creator_support_copy_from() {
        let mut support1 = ProxyCreatorSupport::new();
        support1.add_interface("com.example.MyInterface");

        let mut support2 = ProxyCreatorSupport::new();
        support2.copy_from(&support1);
        assert_eq!(support2.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_debug() {
        let support = ProxyCreatorSupport::new();
        let debug = format!("{:?}", support);
        assert!(!debug.is_empty());
    }

    #[test]
    fn proxy_creator_support_default() {
        let support = ProxyCreatorSupport::default();
        assert_eq!(support.advised().get_advisor_count(), 0);
    }

    #[test]
    fn proxy_creator_support_advised() {
        let support = ProxyCreatorSupport::new();
        let _ = support.advised();
    }

    #[test]
    fn proxy_creator_support_advised_mut() {
        let mut support = ProxyCreatorSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.advised_mut().get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_creator_support_create_proxy_not_implemented() {
        let mut support = ProxyCreatorSupport::new();
        let result = support.create_proxy();
        assert!(result.is_err());
    }
}

