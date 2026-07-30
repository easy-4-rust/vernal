//! groovy_bean_definition_wrapper — 对应 Java 类：org.springframework.beans.factory.groovy.GroovyBeanDefinitionWrapper。
//!
//! 对应 Spring beans.factory.groovy 包。

use std::any::Any;
use std::sync::Arc;

/// GroovyBeanDefinitionWrapper — Spring factory.groovy 组件。
#[derive(Debug, Clone, Default)]
pub struct GroovyBeanDefinitionWrapper {
    // TODO: 添加字段
}

impl GroovyBeanDefinitionWrapper {
    pub fn new() -> Self { Self::default() }
}
