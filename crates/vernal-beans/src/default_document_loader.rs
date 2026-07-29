//! DefaultDocumentLoader — 默认 XML 文档加载器。
use crate::document_loader::{Document, DocumentLoader, Element};

/// 默认 XML 文档加载器。
#[derive(Clone, Debug, Default)]
pub struct DefaultDocumentLoader;
impl DefaultDocumentLoader {
    pub fn new() -> Self { Self }
}
impl DocumentLoader for DefaultDocumentLoader {
    fn load_document(&self, content: &str) -> Result<Document, Box<dyn std::error::Error + Send + Sync>> {
        // Simple stub XML parser
        Ok(Document { root: Some(Element {
            name: "beans".to_string(),
            attributes: std::collections::HashMap::new(),
            children: Vec::new(),
            text: None,
        }), encoding: Some("UTF-8".to_string()) })
    }
}
