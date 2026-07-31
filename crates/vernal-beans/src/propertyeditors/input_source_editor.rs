//! InputSourceEditor — Spring 风格的 InputSource 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.InputSourceEditor`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 InputSource 编辑器。
pub struct InputSourceEditor {
    value: Option<String>,
}

impl InputSourceEditor {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for InputSourceEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for InputSourceEditor {
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

    #[test]
    fn new_editor_has_no_value() {
        let editor = InputSourceEditor::new();
        assert!(editor.value.is_none());
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn default_trait() {
        let editor = InputSourceEditor::default();
        assert!(editor.value.is_none());
    }

    #[test]
    fn set_as_text_stores_value() {
        let mut editor = InputSourceEditor::new();
        editor.set_as_text("http://example.com/data.xml").unwrap();
        assert_eq!(editor.get_as_text(), Some("http://example.com/data.xml".to_string()));
    }

    #[test]
    fn set_as_text_empty_string() {
        let mut editor = InputSourceEditor::new();
        editor.set_as_text("").unwrap();
        assert_eq!(editor.get_as_text(), Some("".to_string()));
    }

    #[test]
    fn set_and_get_value() {
        let mut editor = InputSourceEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new("xml_source".to_string());
        editor.set_value(val);
        let retrieved = editor.get_value().unwrap();
        assert_eq!(*retrieved.downcast_ref::<String>().unwrap(), "xml_source");
    }

    #[test]
    fn set_value_wrong_type_ignored() {
        let mut editor = InputSourceEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn target_and_value_types() {
        let editor = InputSourceEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }
}
