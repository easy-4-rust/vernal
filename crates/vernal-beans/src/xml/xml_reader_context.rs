//! XML 解析模块: xml_reader_context。
use std::any::Any;
use std::sync::Arc;

/// XmlReaderContext — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct XmlReaderContext {
    // TODO: 添加字段
}

impl XmlReaderContext {
    pub fn new() -> Self { Self::default() }
}
