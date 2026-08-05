//! ClassEditor — Spring 风格的类名编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.ClassEditor`。
//! 将字符串转换为类名（在 Rust 中用 String 表示）。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的类名编辑器。
pub struct ClassEditor {
    value: Option<String>,
}

impl ClassEditor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for ClassEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for ClassEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.value = None;
            return Ok(());
        }
        if trimmed.contains(' ') {
            return Err(format!("Invalid class name: {}", text).into());
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

    #[test]
    fn new_editor_has_no_value() {
        let editor = ClassEditor::new();
        assert!(editor.value.is_none());
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn default_trait() {
        let editor = ClassEditor::default();
        assert!(editor.value.is_none());
    }

    #[test]
    fn set_as_text_valid_class_name() {
        let mut editor = ClassEditor::new();
        editor.set_as_text("com.example.MyService").unwrap();
        assert_eq!(
            editor.get_as_text(),
            Some("com.example.MyService".to_string())
        );
    }

    #[test]
    fn set_as_text_trims_whitespace() {
        let mut editor = ClassEditor::new();
        editor.set_as_text("  MyClass  ").unwrap();
        assert_eq!(editor.get_as_text(), Some("MyClass".to_string()));
    }

    #[test]
    fn set_as_text_empty_clears_value() {
        let mut editor = ClassEditor::new();
        editor.set_as_text("MyClass").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_as_text_with_spaces_returns_error() {
        let mut editor = ClassEditor::new();
        let result = editor.set_as_text("invalid class name");
        assert!(result.is_err());
    }

    #[test]
    fn set_and_get_value() {
        let mut editor = ClassEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new("com.example.Service".to_string());
        editor.set_value(val);
        let retrieved = editor.get_value().unwrap();
        assert_eq!(
            *retrieved.downcast_ref::<String>().unwrap(),
            "com.example.Service"
        );
    }

    #[test]
    fn set_value_wrong_type_ignored() {
        let mut editor = ClassEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn target_and_value_types() {
        let editor = ClassEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }
}
