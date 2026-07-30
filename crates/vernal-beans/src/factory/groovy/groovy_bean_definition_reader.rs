//! groovy_bean_definition_reader — 对应 Java 类：org.springframework.beans.factory.groovy.GroovyBeanDefinitionReader。
//!
//! 对应 Spring beans.factory.groovy 包。

use std::any::Any;
use std::sync::Arc;

/// GroovyBeanDefinitionReader — Spring factory.groovy 组件。
#[derive(Debug, Clone, Default)]
pub struct GroovyBeanDefinitionReader {
    // TODO: 添加字段
}

impl GroovyBeanDefinitionReader {
    pub fn new() -> Self { Self::default() }
}
