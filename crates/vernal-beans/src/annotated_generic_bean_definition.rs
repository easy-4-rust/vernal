//! AnnotatedGenericBeanDefinition — Spring 风格的注解驱动通用 Bean 定义。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AnnotatedGenericBeanDefinition`。
//!
//! 结合 GenericBeanDefinition 和 AnnotatedBeanDefinition，用于处理
//! 通过 `@Bean` / `@Configuration` 注解定义的 Bean。

use crate::annotated_bean_definition::AnnotatedBeanDefinition;
use crate::bean_definition::BeanDefinition;
use crate::component_key::ComponentKey;
use crate::component_scope::Scope;
use crate::generic_bean_definition::GenericBeanDefinition;

/// Spring 风格的注解驱动通用 Bean 定义。
///
/// 对应 Spring 的 `AnnotatedGenericBeanDefinition`。
///
/// 结合 GenericBeanDefinition 的完整 Bean 元数据和 AnnotatedBeanDefinition
/// 的注解类型信息，用于处理注解驱动的 Bean 注册。
#[derive(Debug)]
pub struct AnnotatedGenericBeanDefinition {
    /// 内部 GenericBeanDefinition。
    inner: GenericBeanDefinition,
    /// 注解类型名称。
    annotation_type: &'static str,
}

impl AnnotatedGenericBeanDefinition {
    /// 创建指定注解类型的 AnnotatedGenericBeanDefinition。
    pub fn new(annotation_type: &'static str) -> Self {
        Self {
            inner: GenericBeanDefinition::new(),
            annotation_type,
        }
    }

    /// 获取内部的 GenericBeanDefinition 的可变引用。
    pub fn inner_mut(&mut self) -> &mut GenericBeanDefinition {
        &mut self.inner
    }

    /// 获取内部的 GenericBeanDefinition 的引用。
    pub fn inner(&self) -> &GenericBeanDefinition {
        &self.inner
    }
}

impl BeanDefinition for AnnotatedGenericBeanDefinition {
    fn bean_name(&self) -> &ComponentKey {
        unimplemented!("AnnotatedGenericBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        self.inner.get_bean_class_name().unwrap_or("unknown")
    }

    fn scope(&self) -> Scope {
        self.inner.scope()
    }

    fn is_lazy_init(&self) -> bool {
        self.inner.is_lazy_init()
    }

    fn is_primary(&self) -> bool {
        self.inner.is_primary()
    }
}

impl AnnotatedBeanDefinition for AnnotatedGenericBeanDefinition {
    fn annotation_type_name(&self) -> &'static str {
        self.annotation_type
    }
}
