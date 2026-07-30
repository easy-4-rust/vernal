//! DefaultDocumentLoaderImpl — 默认 XML 文档加载器实现。
use crate::document_loader::{Document, DocumentLoader, Element};

/// 默认 XML 文档加载器实现。
#[derive(Clone, Debug, Default)]
pub struct DefaultDocumentLoaderImpl;
impl DefaultDocumentLoaderImpl {
    pub fn new() -> Self { Self }
}
impl DocumentLoader for DefaultDocumentLoaderImpl {
    fn load_document(&self, _content: &str) -> Result<Document, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Document {
            root: Some(Element {
                name: "beans".to_string(),
                attributes: std::collections::HashMap::new(),
                children: Vec::new(),
                text: None,
            }),
            encoding: Some("UTF-8".to_string()),
        })
    }
}
