//! XML 解析模块: default_document_loader。
use std::any::Any;
use std::sync::Arc;

/// DefaultDocumentLoader — 对应 Spring XML 解析组件。
#[derive(Debug, Clone, Default)]
pub struct DefaultDocumentLoader {
    // TODO: 添加字段
}

impl DefaultDocumentLoader {
    pub fn new() -> Self { Self::default() }
}
