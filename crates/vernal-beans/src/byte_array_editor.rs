//! ByteArrayPropertyEditor — Spring 风格的字节数组编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.ByteArrayPropertyEditor`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的字节数组编辑器。
pub struct ByteArrayPropertyEditor {
    value: Option<String>,
}

impl ByteArrayPropertyEditor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for ByteArrayPropertyEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for ByteArrayPropertyEditor {
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
        let editor = ByteArrayPropertyEditor::new();
        assert!(editor.get_as_text().is_none());
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn default_trait_creates_empty_editor() {
        let editor = ByteArrayPropertyEditor::default();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_as_text_stores_value() {
        let mut editor = ByteArrayPropertyEditor::new();
        editor.set_as_text("SGVsbG8=").unwrap();
        assert_eq!(editor.get_as_text(), Some("SGVsbG8=".to_string()));
    }

    #[test]
    fn set_as_text_empty_string() {
        let mut editor = ByteArrayPropertyEditor::new();
        editor.set_as_text("initial").unwrap();
        editor.set_as_text("").unwrap();
        assert_eq!(editor.get_as_text(), Some("".to_string()));
    }

    #[test]
    fn target_type_is_string() {
        let editor = ByteArrayPropertyEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn get_value_type_is_string() {
        let editor = ByteArrayPropertyEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn set_value_with_string() {
        let mut editor = ByteArrayPropertyEditor::new();
        let value: Arc<dyn Any + Send + Sync> = Arc::new("bytes_data".to_string());
        editor.set_value(value);
        assert_eq!(editor.get_as_text(), Some("bytes_data".to_string()));
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = ByteArrayPropertyEditor::new();
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(value);
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn get_value_returns_ref() {
        let mut editor = ByteArrayPropertyEditor::new();
        editor.set_as_text("test_data").unwrap();
        let val = editor.get_value().unwrap();
        assert!(val.is::<String>());
        assert_eq!(val.downcast_ref::<String>().unwrap(), "test_data");
    }

    #[test]
    fn overwrite_previous_value() {
        let mut editor = ByteArrayPropertyEditor::new();
        editor.set_as_text("first").unwrap();
        editor.set_as_text("second").unwrap();
        assert_eq!(editor.get_as_text(), Some("second".to_string()));
    }

    #[test]
    fn binary_like_text() {
        let mut editor = ByteArrayPropertyEditor::new();
        editor.set_as_text("\x00\x01\x02\x03").unwrap();
        assert_eq!(editor.get_as_text(), Some("\x00\x01\x02\x03".to_string()));
    }
}
