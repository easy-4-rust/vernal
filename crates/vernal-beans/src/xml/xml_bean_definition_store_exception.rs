//! XML 解析模块: xml_bean_definition_store_exception。
use std::any::Any;
use std::sync::Arc;

/// XmlBeanDefinitionStoreException — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct XmlBeanDefinitionStoreException {
    // TODO: 添加字段
}

impl XmlBeanDefinitionStoreException {
    pub fn new() -> Self { Self::default() }
}
