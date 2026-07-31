//! 代理工厂。
//!
//! 对应 spring-aop `org.springframework.aop.framework.ProxyFactory`。
//! 用于编程方式创建 AOP 代理的工厂。

use std::sync::Arc;

use crate::Advisor;
use crate::Interceptor;
use crate::target_source::TargetSource;
use super::advised_support::AdvisedSupport;
use super::aop_proxy::AopProxy;

/// 代理工厂。
///
/// 对应 spring-aop `ProxyFactory`。
///
/// 用于编程方式创建 AOP 代理，而不是通过 bean 工厂的声明式设置。
/// 此类提供了一种在自定义用户代码中获取和配置 AOP 代理实例的简单方式。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::ProxyFactory;
///
/// let mut factory = ProxyFactory::new();
/// factory.add_advisor(my_advisor);
/// factory.set_target(my_target);
/// let proxy = factory.get_proxy();
/// ```
pub struct ProxyFactory {
    /// 通知支持类。
    advised: AdvisedSupport,
}

impl ProxyFactory {
    /// 创建新的代理工厂。
    pub fn new() -> Self {
        Self {
            advised: AdvisedSupport::new(),
        }
    }

    /// 创建带目标源的代理工厂。
    pub fn with_target_source(target_source: Arc<dyn TargetSource>) -> Self {
        Self {
            advised: AdvisedSupport::with_target_source(target_source),
        }
    }

    /// 设置目标源。
    pub fn set_target_source(&mut self, target_source: Arc<dyn TargetSource>) {
        self.advised.set_target_source(target_source);
    }

    /// 获取目标源。
    pub fn get_target_source(&self) -> Option<&Arc<dyn TargetSource>> {
        self.advised.get_target_source()
    }

    /// 添加顾问。
    pub fn add_advisor(&mut self, advisor: Arc<Advisor>) {
        self.advised.add_advisor(advisor);
    }

    /// 添加拦截器作为顾问。
    pub fn add_interceptor(&mut self, interceptor: Arc<dyn Interceptor>) {
        let advisor = Arc::new(Advisor::shared(
            Arc::new(crate::any_pointcut::AnyPointcut::new()),
            interceptor,
            0,
        ));
        self.advised.add_advisor(advisor);
    }

    /// 获取所有顾问。
    pub fn get_advisors(&self) -> &[Arc<Advisor>] {
        self.advised.get_advisors()
    }

    /// 添加接口。
    pub fn add_interface(&mut self, interface: impl Into<String>) {
        self.advised.add_interface(interface);
    }

    /// 获取所有接口。
    pub fn get_interfaces(&self) -> &[String] {
        self.advised.get_interfaces()
    }

    /// 设置是否代理目标类。
    pub fn set_proxy_target_class(&mut self, proxy_target_class: bool) {
        self.advised.config_mut().set_proxy_target_class(proxy_target_class);
    }

    /// 是否代理目标类。
    pub fn is_proxy_target_class(&self) -> bool {
        self.advised.config().is_proxy_target_class()
    }

    /// 冻结配置。
    pub fn freeze(&mut self) {
        self.advised.freeze();
    }

    /// 是否冻结。
    pub fn is_frozen(&self) -> bool {
        self.advised.is_frozen()
    }

    /// 获取通知支持类。
    pub fn advised(&self) -> &AdvisedSupport {
        &self.advised
    }

    /// 获取可变通知支持类。
    pub fn advised_mut(&mut self) -> &mut AdvisedSupport {
        &mut self.advised
    }

    /// 创建代理。
    ///
    /// 注意：此方法需要 vernal-beans 完成后才能完整实现。
    /// 当前返回一个占位符。
    pub fn get_proxy(&self) -> Result<Box<dyn AopProxy>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: 需要 vernal-beans 完成后实现
        Err("ProxyFactory.get_proxy() not implemented yet - requires vernal-beans".into())
    }
}

impl Default for ProxyFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ProxyFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyFactory")
            .field("advised", &self.advised)
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
    fn proxy_factory_creation() {
        let factory = ProxyFactory::new();
        assert_eq!(factory.get_advisors().len(), 0);
        assert_eq!(factory.get_interfaces().len(), 0);
        assert!(!factory.is_proxy_target_class());
        assert!(!factory.is_frozen());
    }

    #[test]
    fn proxy_factory_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let factory = ProxyFactory::with_target_source(target_source);
        assert!(factory.get_target_source().is_some());
    }

    #[test]
    fn proxy_factory_add_interface() {
        let mut factory = ProxyFactory::new();
        factory.add_interface("com.example.MyInterface");
        assert_eq!(factory.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_factory_set_proxy_target_class() {
        let mut factory = ProxyFactory::new();
        factory.set_proxy_target_class(true);
        assert!(factory.is_proxy_target_class());
    }

    #[test]
    fn proxy_factory_freeze() {
        let mut factory = ProxyFactory::new();
        assert!(!factory.is_frozen());
        factory.freeze();
        assert!(factory.is_frozen());
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
    fn proxy_factory_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let factory = ProxyFactory::with_target_source(target_source);
        assert!(factory.get_target_source().is_some());
    }

    #[test]
    fn proxy_factory_set_target_source() {
        let mut factory = ProxyFactory::new();
        let target_source = Arc::new(TestTargetSource);
        factory.set_target_source(target_source);
        assert!(factory.get_target_source().is_some());
    }

    #[test]
    fn proxy_factory_add_advisor() {
        let mut factory = ProxyFactory::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        factory.add_advisor(advisor);
        assert_eq!(factory.get_advisors().len(), 1);
    }

    #[test]
    fn proxy_factory_add_interceptor() {
        let mut factory = ProxyFactory::new();
        factory.add_interceptor(Arc::new(TestInterceptor));
        assert_eq!(factory.get_advisors().len(), 1);
    }

    #[test]
    fn proxy_factory_add_interface() {
        let mut factory = ProxyFactory::new();
        factory.add_interface("com.example.MyInterface");
        assert_eq!(factory.get_interfaces().len(), 1);
    }

    #[test]
    fn proxy_factory_set_proxy_target_class() {
        let mut factory = ProxyFactory::new();
        factory.set_proxy_target_class(true);
        assert!(factory.is_proxy_target_class());
    }

    #[test]
    fn proxy_factory_freeze() {
        let mut factory = ProxyFactory::new();
        factory.freeze();
        assert!(factory.is_frozen());
    }

    #[test]
    fn proxy_factory_advised() {
        let factory = ProxyFactory::new();
        let _advised = factory.advised();
    }

    #[test]
    fn proxy_factory_advised_mut() {
        let mut factory = ProxyFactory::new();
        let _advised = factory.advised_mut();
    }

    #[test]
    fn proxy_factory_debug() {
        let factory = ProxyFactory::new();
        let debug = format!("{:?}", factory);
        assert!(!debug.is_empty());
    }

    #[test]
    fn proxy_factory_default() {
        let factory = ProxyFactory::default();
        assert_eq!(factory.get_advisors().len(), 0);
    }
}
