//! XML 解析模块: default_bean_definition_document_reader。
use std::any::Any;
use std::sync::Arc;

/// DefaultBeanDefinitionDocumentReader — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct DefaultBeanDefinitionDocumentReader {
    // TODO: 添加字段
}

impl DefaultBeanDefinitionDocumentReader {
    pub fn new() -> Self { Self::default() }
}
