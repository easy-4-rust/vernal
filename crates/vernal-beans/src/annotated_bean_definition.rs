//! AnnotatedBeanDefinition — Spring 风格的注解驱动 Bean 定义 trait。
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AnnotatedBeanDefinition`。
use std::any::Any;
use crate::bean_definition::BeanDefinition;

/// Spring 风格的注解驱动 Bean 定义 trait。
pub trait AnnotatedBeanDefinition: BeanDefinition {
    fn annotation_metadata(&self) -> &dyn Any;
}
