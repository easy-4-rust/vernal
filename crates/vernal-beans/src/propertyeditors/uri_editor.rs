//! URIEditor — Spring 风格的 URI 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.URIEditor`。
//!
//! 将字符串转换为 `std::net::Uri`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 URI 编辑器。
///
/// 对应 Spring 的 `URIEditor`。
pub struct URIEditor {
    value: Option<String>,
}

impl URIEditor {
    /// 创建 URI 编辑器。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for URIEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for URIEditor {
    fn target_type(&self) -> std::any::TypeId {
        // URI 在 Rust 中没有统一类型，用 String 表示
        std::any::TypeId::of::<String>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.value = None;
            return Ok(());
        }
        // 验证 URI 格式（简单验证）
        if !trimmed.contains(':') && !trimmed.starts_with('/') {
            return Err(format!("Invalid URI: {}", text).into());
        }
        self.value = Some(trimmed.to_string());
        Ok(())
    }

    fn get_as_text(&self) -> Option<String> {
        self.value.clone()
    }

    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(s) = value.downcast_ref::<String>() {
            self.value = Some(s.clone());
        }
    }

    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_ref().map(|v| v as &dyn Any)
    }

    fn get_value_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_editor::PropertyEditor;

    #[test]
    fn new_editor_has_no_value() {
        let editor = URIEditor::new();
        assert!(editor.get_as_text().is_none());
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn default_trait_creates_empty() {
        let editor = URIEditor::default();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn target_type_is_string() {
        let editor = URIEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn get_value_type_is_string() {
        let editor = URIEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn set_as_text_valid_uri_with_colon() {
        let mut editor = URIEditor::new();
        editor.set_as_text("https://example.com").unwrap();
        assert_eq!(editor.get_as_text(), Some("https://example.com".to_string()));
    }

    #[test]
    fn set_as_text_valid_uri_with_slash() {
        let mut editor = URIEditor::new();
        editor.set_as_text("/path/to/resource").unwrap();
        assert_eq!(editor.get_as_text(), Some("/path/to/resource".to_string()));
    }

    #[test]
    fn set_as_text_empty_string_sets_none() {
        let mut editor = URIEditor::new();
        editor.set_as_text("https://example.com").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_as_text_whitespace_only_sets_none() {
        let mut editor = URIEditor::new();
        editor.set_as_text("  ").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_as_text_invalid_uri_returns_error() {
        let mut editor = URIEditor::new();
        let result = editor.set_as_text("notauri");
        assert!(result.is_err());
    }

    #[test]
    fn set_as_text_trims_whitespace() {
        let mut editor = URIEditor::new();
        editor.set_as_text("  https://example.com  ").unwrap();
        assert_eq!(editor.get_as_text(), Some("https://example.com".to_string()));
    }

    #[test]
    fn set_value_with_string() {
        let mut editor = URIEditor::new();
        editor.set_value(Arc::new("https://test.com".to_string()));
        assert_eq!(editor.get_as_text(), Some("https://test.com".to_string()));
    }

    #[test]
    fn set_value_with_non_string_ignored() {
        let mut editor = URIEditor::new();
        editor.set_value(Arc::new(42i32));
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn get_value_returns_string_ref() {
        let mut editor = URIEditor::new();
        editor.set_as_text("https://example.com").unwrap();
        let value = editor.get_value().unwrap();
        assert!(value.downcast_ref::<String>().is_some());
    }

    #[test]
    fn set_as_text_ftp_uri() {
        let mut editor = URIEditor::new();
        editor.set_as_text("ftp://files.example.com").unwrap();
        assert_eq!(editor.get_as_text(), Some("ftp://files.example.com".to_string()));
    }

    #[test]
    fn set_as_text_mailto_uri() {
        let mut editor = URIEditor::new();
        editor.set_as_text("mailto:user@example.com").unwrap();
        assert_eq!(editor.get_as_text(), Some("mailto:user@example.com".to_string()));
    }

    #[test]
    fn set_as_text_file_uri() {
        let mut editor = URIEditor::new();
        editor.set_as_text("file:///path/to/file").unwrap();
        assert_eq!(editor.get_as_text(), Some("file:///path/to/file".to_string()));
    }
}
