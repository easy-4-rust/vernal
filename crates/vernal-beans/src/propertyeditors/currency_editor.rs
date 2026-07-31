//! CurrencyEditor — Spring 风格的 Currency 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CurrencyEditor`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 Currency 编辑器。
pub struct CurrencyEditor {
    value: Option<String>,
}

impl CurrencyEditor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for CurrencyEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for CurrencyEditor {
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
        let editor = CurrencyEditor::new();
        assert!(editor.value.is_none());
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn default_trait() {
        let editor = CurrencyEditor::default();
        assert!(editor.value.is_none());
    }

    #[test]
    fn set_as_text_stores_value() {
        let mut editor = CurrencyEditor::new();
        editor.set_as_text("USD").unwrap();
        assert_eq!(editor.get_as_text(), Some("USD".to_string()));
    }

    #[test]
    fn set_as_text_trims_whitespace() {
        let mut editor = CurrencyEditor::new();
        editor.set_as_text("  EUR  ").unwrap();
        assert_eq!(editor.get_as_text(), Some("EUR".to_string()));
    }

    #[test]
    fn set_as_text_empty_clears_value() {
        let mut editor = CurrencyEditor::new();
        editor.set_as_text("USD").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_and_get_value() {
        let mut editor = CurrencyEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new("GBP".to_string());
        editor.set_value(val);
        let retrieved = editor.get_value().unwrap();
        assert_eq!(*retrieved.downcast_ref::<String>().unwrap(), "GBP");
    }

    #[test]
    fn set_value_wrong_type_ignored() {
        let mut editor = CurrencyEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn target_and_value_types() {
        let editor = CurrencyEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }
}
