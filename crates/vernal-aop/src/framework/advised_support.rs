//! 通知支持类。
//!
//! 对应 spring-aop `org.springframework.aop.framework.AdvisedSupport`。
//! AOP 代理配置管理器的基类。

use std::sync::Arc;

use super::proxy_config::ProxyConfig;
use crate::Advisor;
use crate::target_source::TargetSource;

/// 通知支持类。
///
/// 对应 spring-aop `AdvisedSupport`。
///
/// AOP 代理配置管理器的基类。这些本身不是 AOP 代理，
/// 但此类的子类通常是直接获取 AOP 代理实例的工厂。
///
/// # 特性
///
/// - 管理通知和顾问
/// - 持有目标源
/// - 管理代理接口
/// - 支持配置冻结
pub struct AdvisedSupport {
    /// 代理配置。
    config: ProxyConfig,
    /// 目标源。
    target_source: Option<Arc<dyn TargetSource>>,
    /// 顾问列表。
    advisors: Vec<Arc<Advisor>>,
    /// 代理接口。
    interfaces: Vec<String>,
    /// 是否预过滤。
    pre_filtered: bool,
}

impl AdvisedSupport {
    /// 创建新的通知支持类。
    pub fn new() -> Self {
        Self {
            config: ProxyConfig::new(),
            target_source: None,
            advisors: Vec::new(),
            interfaces: Vec::new(),
            pre_filtered: false,
        }
    }

    /// 创建带目标源的通知支持类。
    pub fn with_target_source(target_source: Arc<dyn TargetSource>) -> Self {
        Self {
            config: ProxyConfig::new(),
            target_source: Some(target_source),
            advisors: Vec::new(),
            interfaces: Vec::new(),
            pre_filtered: false,
        }
    }

    /// 获取代理配置。
    pub fn config(&self) -> &ProxyConfig {
        &self.config
    }

    /// 获取可变代理配置。
    pub fn config_mut(&mut self) -> &mut ProxyConfig {
        &mut self.config
    }

    /// 设置目标源。
    pub fn set_target_source(&mut self, target_source: Arc<dyn TargetSource>) {
        self.target_source = Some(target_source);
    }

    /// 获取目标源。
    pub fn get_target_source(&self) -> Option<&Arc<dyn TargetSource>> {
        self.target_source.as_ref()
    }

    /// 添加顾问。
    pub fn add_advisor(&mut self, advisor: Arc<Advisor>) {
        self.advisors.push(advisor);
    }

    /// 在指定位置添加顾问。
    pub fn add_advisor_at(&mut self, index: usize, advisor: Arc<Advisor>) {
        self.advisors.insert(index, advisor);
    }

    /// 移除顾问。
    pub fn remove_advisor(&mut self, advisor: &Arc<Advisor>) -> bool {
        let len = self.advisors.len();
        self.advisors.retain(|a| !Arc::ptr_eq(a, advisor));
        self.advisors.len() < len
    }

    /// 移除指定位置的顾问。
    pub fn remove_advisor_at(&mut self, index: usize) -> Arc<Advisor> {
        self.advisors.remove(index)
    }

    /// 获取所有顾问。
    pub fn get_advisors(&self) -> &[Arc<Advisor>] {
        &self.advisors
    }

    /// 获取顾问数量。
    pub fn get_advisor_count(&self) -> usize {
        self.advisors.len()
    }

    /// 添加接口。
    pub fn add_interface(&mut self, interface: impl Into<String>) {
        self.interfaces.push(interface.into());
    }

    /// 移除接口。
    pub fn remove_interface(&mut self, interface: &str) -> bool {
        let len = self.interfaces.len();
        self.interfaces.retain(|i| i != interface);
        self.interfaces.len() < len
    }

    /// 获取所有接口。
    pub fn get_interfaces(&self) -> &[String] {
        &self.interfaces
    }

    /// 设置是否预过滤。
    pub fn set_pre_filtered(&mut self, pre_filtered: bool) {
        self.pre_filtered = pre_filtered;
    }

    /// 是否预过滤。
    pub fn is_pre_filtered(&self) -> bool {
        self.pre_filtered
    }

    /// 冻结配置。
    pub fn freeze(&mut self) {
        self.config.set_frozen(true);
    }

    /// 是否冻结。
    pub fn is_frozen(&self) -> bool {
        self.config.is_frozen()
    }

    /// 从另一个通知支持类复制配置。
    pub fn copy_from(&mut self, other: &AdvisedSupport) {
        self.config.copy_from(&other.config);
        self.target_source = other.target_source.clone();
        self.advisors = other.advisors.clone();
        self.interfaces = other.interfaces.clone();
        self.pre_filtered = other.pre_filtered;
    }

    /// 获取顾问链。
    pub fn get_interceptors_and_dynamic_interception_advice(
        &self,
    ) -> Vec<Arc<dyn crate::Interceptor>> {
        let mut interceptors = Vec::new();
        for advisor in &self.advisors {
            interceptors.push(advisor.interceptor());
        }
        interceptors
    }
}

impl Default for AdvisedSupport {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AdvisedSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdvisedSupport")
            .field("config", &self.config)
            .field("target_source", &self.target_source.is_some())
            .field("advisor_count", &self.advisors.len())
            .field("interfaces", &self.interfaces)
            .field("pre_filtered", &self.pre_filtered)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;
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
    fn advised_support_creation() {
        let support = AdvisedSupport::new();
        assert_eq!(support.get_advisor_count(), 0);
        assert_eq!(support.get_interfaces().len(), 0);
        assert!(!support.is_pre_filtered());
        assert!(!support.is_frozen());
    }

    #[test]
    fn advised_support_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let support = AdvisedSupport::with_target_source(target_source);
        assert!(support.get_target_source().is_some());
    }

    #[test]
    fn advised_support_add_advisor() {
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

        let mut support = AdvisedSupport::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        support.add_advisor(advisor);
        assert_eq!(support.get_advisor_count(), 1);
    }

    #[test]
    fn advised_support_add_interface() {
        let mut support = AdvisedSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
        assert_eq!(support.get_interfaces()[0], "com.example.MyInterface");
    }

    #[test]
    fn advised_support_freeze() {
        let mut support = AdvisedSupport::new();
        assert!(!support.is_frozen());
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
        fn intercept<'a>(
            &'a self,
            invocation: Arc<crate::Invocation>,
            next: crate::Next<'a>,
        ) -> crate::InvocationFuture<'a> {
            next.run(invocation)
        }
    }

    struct TestTargetSource;
    impl crate::target_source::TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            Some("TestTarget")
        }
        fn get_target(
            &self,
        ) -> Result<Box<dyn std::any::Any>, crate::target_source_error::TargetSourceError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn advised_support_set_target_source() {
        let mut support = AdvisedSupport::new();
        assert!(support.get_target_source().is_none());
        let target_source = Arc::new(TestTargetSource);
        support.set_target_source(target_source);
        assert!(support.get_target_source().is_some());
    }

    #[test]
    fn advised_support_remove_advisor() {
        let mut support = AdvisedSupport::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        support.add_advisor(advisor.clone());
        assert_eq!(support.get_advisor_count(), 1);
        support.remove_advisor(&advisor);
        assert_eq!(support.get_advisor_count(), 0);
    }

    #[test]
    fn advised_support_remove_advisor_at() {
        let mut support = AdvisedSupport::new();
        support.add_advisor(Arc::new(crate::Advisor::new(
            AnyPointcut::new(),
            TestInterceptor,
            0,
        )));
        support.add_advisor(Arc::new(crate::Advisor::new(
            AnyPointcut::new(),
            TestInterceptor,
            1,
        )));
        assert_eq!(support.get_advisor_count(), 2);
        support.remove_advisor_at(0);
        assert_eq!(support.get_advisor_count(), 1);
    }

    #[test]
    fn advised_support_remove_interface() {
        let mut support = AdvisedSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
        support.remove_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 0);
    }

    #[test]
    fn advised_support_set_pre_filtered() {
        let mut support = AdvisedSupport::new();
        assert!(!support.is_pre_filtered());
        support.set_pre_filtered(true);
        assert!(support.is_pre_filtered());
    }

    #[test]
    fn advised_support_config() {
        let support = AdvisedSupport::new();
        let config = support.config();
        assert!(!config.is_proxy_target_class());
    }

    #[test]
    fn advised_support_config_mut() {
        let mut support = AdvisedSupport::new();
        support.config_mut().set_proxy_target_class(true);
        assert!(support.config().is_proxy_target_class());
    }

    #[test]
    fn advised_support_copy_from() {
        let mut support1 = AdvisedSupport::new();
        support1.add_interface("com.example.MyInterface");
        support1.config_mut().set_proxy_target_class(true);

        let mut support2 = AdvisedSupport::new();
        support2.copy_from(&support1);
        assert_eq!(support2.get_interfaces().len(), 1);
        assert!(support2.config().is_proxy_target_class());
    }

    #[test]
    fn advised_support_debug() {
        let support = AdvisedSupport::new();
        let debug = format!("{:?}", support);
        assert!(!debug.is_empty());
    }

    #[test]
    fn advised_support_default() {
        let support = AdvisedSupport::default();
        assert_eq!(support.get_advisor_count(), 0);
    }
}

#[cfg(test)]
mod advised_support_coverage_tests {
    use super::*;
    use crate::any_pointcut::AnyPointcut;

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

    struct TestTargetSource;
    impl crate::target_source::TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            Some("TestTarget")
        }
        fn get_target(
            &self,
        ) -> Result<Box<dyn std::any::Any>, crate::target_source_error::TargetSourceError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn advised_support_new() {
        let support = AdvisedSupport::new();
        assert_eq!(support.get_advisor_count(), 0);
        assert_eq!(support.get_interfaces().len(), 0);
        assert!(!support.is_pre_filtered());
        assert!(!support.is_frozen());
    }

    #[test]
    fn advised_support_with_target_source() {
        let target_source = Arc::new(TestTargetSource);
        let support = AdvisedSupport::with_target_source(target_source);
        assert!(support.get_target_source().is_some());
    }

    #[test]
    fn advised_support_add_advisor() {
        let mut support = AdvisedSupport::new();
        let advisor = Arc::new(crate::Advisor::new(AnyPointcut::new(), TestInterceptor, 0));
        support.add_advisor(advisor);
        assert_eq!(support.get_advisor_count(), 1);
    }

    #[test]
    fn advised_support_add_interface() {
        let mut support = AdvisedSupport::new();
        support.add_interface("com.example.MyInterface");
        assert_eq!(support.get_interfaces().len(), 1);
        assert_eq!(support.get_interfaces()[0], "com.example.MyInterface");
    }

    #[test]
    fn advised_support_freeze() {
        let mut support = AdvisedSupport::new();
        assert!(!support.is_frozen());
        support.freeze();
        assert!(support.is_frozen());
    }

    #[test]
    fn advised_support_default() {
        let support = AdvisedSupport::default();
        assert_eq!(support.get_advisor_count(), 0);
    }

    #[test]
    fn advised_support_debug() {
        let support = AdvisedSupport::new();
        let debug = format!("{:?}", support);
        assert!(!debug.is_empty());
    }

    #[test]
    fn advised_support_copy_from() {
        let mut support1 = AdvisedSupport::new();
        support1.add_interface("com.example.MyInterface");
        support1.config_mut().set_proxy_target_class(true);

        let mut support2 = AdvisedSupport::new();
        support2.copy_from(&support1);
        assert_eq!(support2.get_interfaces().len(), 1);
        assert!(support2.config().is_proxy_target_class());
    }

    #[test]
    fn advised_support_get_interceptors_and_dynamic_interception_advice() {
        let mut support = AdvisedSupport::new();
        support.add_advisor(Arc::new(crate::Advisor::new(
            AnyPointcut::new(),
            TestInterceptor,
            0,
        )));
        let interceptors = support.get_interceptors_and_dynamic_interception_advice();
        assert_eq!(interceptors.len(), 1);
    }
}
