//! CharArrayPropertyEditor — Spring 风格的字符数组属性编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CharArrayPropertyEditor`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的字符数组属性编辑器。
pub struct CharArrayPropertyEditor {
    value: Option<String>,
}

impl CharArrayPropertyEditor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for CharArrayPropertyEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for CharArrayPropertyEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.value = Some(text.to_string());
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
        let editor = CharArrayPropertyEditor::new();
        assert!(editor.get_as_text().is_none());
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn default_trait_creates_empty() {
        let editor = CharArrayPropertyEditor::default();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn target_type_is_string() {
        let editor = CharArrayPropertyEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn get_value_type_is_string() {
        let editor = CharArrayPropertyEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn set_as_text_basic() {
        let mut editor = CharArrayPropertyEditor::new();
        editor.set_as_text("hello").unwrap();
        assert_eq!(editor.get_as_text(), Some("hello".to_string()));
    }

    #[test]
    fn set_as_text_empty_string() {
        let mut editor = CharArrayPropertyEditor::new();
        editor.set_as_text("hello").unwrap();
        editor.set_as_text("").unwrap();
        assert_eq!(editor.get_as_text(), Some("".to_string()));
    }

    #[test]
    fn set_as_text_special_characters() {
        let mut editor = CharArrayPropertyEditor::new();
        editor.set_as_text("hello\nworld\t!").unwrap();
        assert_eq!(editor.get_as_text(), Some("hello\nworld\t!".to_string()));
    }

    #[test]
    fn set_as_text_overwrites_previous() {
        let mut editor = CharArrayPropertyEditor::new();
        editor.set_as_text("first").unwrap();
        editor.set_as_text("second").unwrap();
        assert_eq!(editor.get_as_text(), Some("second".to_string()));
    }

    #[test]
    fn set_value_with_string() {
        let mut editor = CharArrayPropertyEditor::new();
        editor.set_value(Arc::new("from_value".to_string()));
        assert_eq!(editor.get_as_text(), Some("from_value".to_string()));
    }

    #[test]
    fn set_value_with_non_string_ignored() {
        let mut editor = CharArrayPropertyEditor::new();
        editor.set_value(Arc::new(42i32));
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn get_value_returns_string_ref() {
        let mut editor = CharArrayPropertyEditor::new();
        editor.set_as_text("test").unwrap();
        let value = editor.get_value().unwrap();
        assert!(value.downcast_ref::<String>().is_some());
        assert_eq!(*value.downcast_ref::<String>().unwrap(), "test");
    }

    #[test]
    fn set_as_text_unicode() {
        let mut editor = CharArrayPropertyEditor::new();
        editor.set_as_text("hello world").unwrap();
        assert_eq!(editor.get_as_text(), Some("hello world".to_string()));
    }

    #[test]
    fn set_as_text_whitespace_preserved() {
        let mut editor = CharArrayPropertyEditor::new();
        editor.set_as_text("  spaces  ").unwrap();
        assert_eq!(editor.get_as_text(), Some("  spaces  ".to_string()));
    }
}
