//! PatternEditor — Spring 风格的正则表达式模式编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.PatternEditor`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 Pattern 编辑器。
pub struct PatternEditor {
    value: Option<String>,
}

impl PatternEditor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for PatternEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for PatternEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.value = None;
            return Ok(());
        }
        // 简单验证：正则表达式不应为空
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

    #[test]
    fn new_editor_has_no_value() {
        let editor = PatternEditor::new();
        assert!(editor.value.is_none());
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn default_trait() {
        let editor = PatternEditor::default();
        assert!(editor.value.is_none());
    }

    #[test]
    fn set_as_text_stores_value() {
        let mut editor = PatternEditor::new();
        editor.set_as_text("^\\d+$").unwrap();
        assert_eq!(editor.get_as_text(), Some("^\\d+$".to_string()));
    }

    #[test]
    fn set_as_text_trims_whitespace() {
        let mut editor = PatternEditor::new();
        editor.set_as_text("  [a-z]+  ").unwrap();
        assert_eq!(editor.get_as_text(), Some("[a-z]+".to_string()));
    }

    #[test]
    fn set_as_text_empty_clears_value() {
        let mut editor = PatternEditor::new();
        editor.set_as_text("abc").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_and_get_value() {
        let mut editor = PatternEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new("\\w+".to_string());
        editor.set_value(val);
        let retrieved = editor.get_value().unwrap();
        assert_eq!(*retrieved.downcast_ref::<String>().unwrap(), "\\w+");
    }

    #[test]
    fn set_value_wrong_type_ignored() {
        let mut editor = PatternEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn target_and_value_types() {
        let editor = PatternEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }
}
