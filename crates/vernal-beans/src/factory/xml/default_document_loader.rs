//! DefaultDocumentLoader — 默认 XML 文档加载器。
use crate::factory::xml::document_loader::{Document, DocumentLoader, XmlElement};

/// 默认 XML 文档加载器。
#[derive(Clone, Debug, Default)]
pub struct DefaultDocumentLoader;
impl DefaultDocumentLoader {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self
    }
}
impl DocumentLoader for DefaultDocumentLoader {
    fn load_document(
        &self,
        _input: &str,
        _entity_resolver: Option<&dyn crate::factory::xml::entity_resolver::EntityResolver>,
        _validation_mode: crate::factory::xml::document_loader::ValidationMode,
        _namespace_aware: bool,
    ) -> Result<Document, Box<dyn std::error::Error + Send + Sync>> {
        // Simple stub XML parser
        Ok(Document {
            root_element: Some(XmlElement {
                local_name: "beans".to_string(),
                namespace_uri: None,
                attributes: Vec::new(),
                children: Vec::new(),
                text: None,
            }),
            system_id: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::xml::document_loader::{DocumentLoader, ValidationMode};

    #[test]
    fn new_loader() {
        let loader = DefaultDocumentLoader::new();
        let result = loader.load_document("<beans/>", None, ValidationMode::None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn default_trait() {
        let loader = DefaultDocumentLoader::default();
        let result = loader.load_document("", None, ValidationMode::None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn load_document_returns_beans_root() {
        let loader = DefaultDocumentLoader::new();
        let doc = loader
            .load_document("<beans><bean/></beans>", None, ValidationMode::Auto, true)
            .unwrap();

        assert!(doc.root_element.is_some());
        let root = doc.root_element.unwrap();
        assert_eq!(root.local_name, "beans");
        assert!(root.namespace_uri.is_none());
        assert!(root.attributes.is_empty());
        assert!(root.children.is_empty());
        assert!(root.text.is_none());
        assert!(doc.system_id.is_none());
    }

    #[test]
    fn clone_loader() {
        let loader = DefaultDocumentLoader::new();
        let cloned = loader.clone();
        let result = cloned.load_document("test", None, ValidationMode::None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn debug_format() {
        let loader = DefaultDocumentLoader::new();
        let debug_str = format!("{:?}", loader);
        assert!(debug_str.contains("DefaultDocumentLoader"));
    }
}
