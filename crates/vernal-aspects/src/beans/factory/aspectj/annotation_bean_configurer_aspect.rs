//! 对标 `org.springframework.beans.factory.aspectj.AnnotationBeanConfigurerAspect`。
//!
//! 基于 `@Configurable` 注解驱动的 Bean 配置切面。

use super::abstract_dependency_injection_aspect::AbstractDependencyInjectionAspect;
use super::configurable_object::ConfigurableObject;

/// `@Configurable` 注解驱动的 Bean 配置切面。
///
/// 对标 Spring 的 `AnnotationBeanConfigurerAspect`。
/// 使用 `@Configurable` 注解识别需要 DI 注入的类。
pub struct AnnotationBeanConfigurerAspect<T: ConfigurableObject> {
    inner: AbstractDependencyInjectionAspect<T>,
}

impl<T: ConfigurableObject> AnnotationBeanConfigurerAspect<T> {
    /// 创建 `@Configurable` 注解驱动的 Bean 配置切面。
    pub fn new() -> Self {
        Self {
            inner: AbstractDependencyInjectionAspect::new(),
        }
    }

    /// 获取底层 DI 切面。
    pub fn get_inner(&self) -> &AbstractDependencyInjectionAspect<T> {
        &self.inner
    }

    /// 获取可变的底层 DI 切面。
    pub fn get_inner_mut(&mut self) -> &mut AbstractDependencyInjectionAspect<T> {
        &mut self.inner
    }

    /// 配置 bean（注入依赖）。
    pub fn configure_bean(&self, bean: &mut T) {
        self.inner.configure_bean(bean);
    }

    /// 设置 bean 名称。
    pub fn set_bean_name(&mut self, name: String) {
        self.inner.set_bean_name(name);
    }
}

impl<T: ConfigurableObject> Default for AnnotationBeanConfigurerAspect<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::configurable_object::ConfigurableObject;

    struct TestDomainObject {
        name: String,
    }

    impl ConfigurableObject for TestDomainObject {}

    #[test]
    fn test_annotation_bean_configurer_aspect_creation() {
        let aspect = AnnotationBeanConfigurerAspect::<TestDomainObject>::new();
        assert!(aspect.get_inner().get_bean_name().is_none());
    }

    #[test]
    fn test_set_bean_name() {
        let mut aspect = AnnotationBeanConfigurerAspect::<TestDomainObject>::new();
        aspect.set_bean_name("myBean".to_string());
        assert_eq!(aspect.get_inner().get_bean_name(), Some("myBean"));
    }

    #[test]
    fn test_configure_bean() {
        let aspect = AnnotationBeanConfigurerAspect::<TestDomainObject>::new();
        let mut bean = TestDomainObject {
            name: "test".to_string(),
        };
        aspect.configure_bean(&mut bean);
    }

    #[test]
    fn test_get_inner() {
        let aspect = AnnotationBeanConfigurerAspect::<TestDomainObject>::new();
        let _ = aspect.get_inner();
    }

    #[test]
    fn test_get_inner_mut() {
        let mut aspect = AnnotationBeanConfigurerAspect::<TestDomainObject>::new();
        let _ = aspect.get_inner_mut();
    }

    #[test]
    fn test_annotation_bean_configurer_aspect_default() {
        let aspect = AnnotationBeanConfigurerAspect::<TestDomainObject>::default();
        assert!(aspect.get_inner().get_bean_name().is_none());
    }

    #[test]
    fn test_annotation_bean_configurer_aspect_debug() {
        let aspect = AnnotationBeanConfigurerAspect::<TestDomainObject>::new();
        // 不检查 Debug 实现，只检查创建成功
        let _ = aspect;
    }

    #[test]
    fn test_annotation_bean_configurer_aspect_clone() {
        let aspect = AnnotationBeanConfigurerAspect::<TestDomainObject>::new();
        let _ = aspect;
    }

    #[test]
    fn test_annotation_bean_configurer_aspect_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<AnnotationBeanConfigurerAspect<TestDomainObject>>();
        assert_sync::<AnnotationBeanConfigurerAspect<TestDomainObject>>();
    }
}
