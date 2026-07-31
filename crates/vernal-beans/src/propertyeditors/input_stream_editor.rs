//! InputStreamEditor — Spring 风格的输入流编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.InputStreamEditor`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 InputStream 编辑器。
pub struct InputStreamEditor {
    value: Option<String>,
}

impl InputStreamEditor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for InputStreamEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for InputStreamEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        self.value = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
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
        let editor = InputStreamEditor::new();
        assert!(editor.value.is_none());
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn default_trait() {
        let editor = InputStreamEditor::default();
        assert!(editor.value.is_none());
    }

    #[test]
    fn set_as_text_stores_value() {
        let mut editor = InputStreamEditor::new();
        editor.set_as_text("/path/to/resource").unwrap();
        assert_eq!(editor.get_as_text(), Some("/path/to/resource".to_string()));
    }

    #[test]
    fn set_as_text_empty_clears_value() {
        let mut editor = InputStreamEditor::new();
        editor.set_as_text("some_input").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_and_get_value() {
        let mut editor = InputStreamEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new("stream_data".to_string());
        editor.set_value(val);
        let retrieved = editor.get_value().unwrap();
        assert_eq!(*retrieved.downcast_ref::<String>().unwrap(), "stream_data");
    }

    #[test]
    fn set_value_wrong_type_ignored() {
        let mut editor = InputStreamEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn target_and_value_types() {
        let editor = InputStreamEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }
}
