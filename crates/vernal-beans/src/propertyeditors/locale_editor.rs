//! LocaleEditor — Spring 风格的 Locale 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.LocaleEditor`。
//! 将字符串转换为 locale 标识（用 String 表示）。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 Locale 编辑器。
pub struct LocaleEditor {
    value: Option<String>,
}

impl LocaleEditor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for LocaleEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for LocaleEditor {
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
        let editor = LocaleEditor::new();
        assert!(editor.get_as_text().is_none());
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn default_creates_empty_editor() {
        let editor = LocaleEditor::default();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_valid_locale() {
        let mut editor = LocaleEditor::new();
        editor.set_as_text("en_US").unwrap();
        assert_eq!(editor.get_as_text(), Some("en_US".to_string()));
    }

    #[test]
    fn set_with_whitespace_trimmed() {
        let mut editor = LocaleEditor::new();
        editor.set_as_text("  zh_CN  ").unwrap();
        assert_eq!(editor.get_as_text(), Some("zh_CN".to_string()));
    }

    #[test]
    fn empty_string_sets_none() {
        let mut editor = LocaleEditor::new();
        editor.set_as_text("en").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn whitespace_only_sets_none() {
        let mut editor = LocaleEditor::new();
        editor.set_as_text("   ").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn target_type_is_string() {
        let editor = LocaleEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn get_value_type_is_string() {
        let editor = LocaleEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn set_value_with_string() {
        let mut editor = LocaleEditor::new();
        let value: Arc<dyn Any + Send + Sync> = Arc::new("fr_FR".to_string());
        editor.set_value(value);
        assert_eq!(editor.get_as_text(), Some("fr_FR".to_string()));
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = LocaleEditor::new();
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(value);
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn get_value_returns_ref() {
        let mut editor = LocaleEditor::new();
        editor.set_as_text("de_DE").unwrap();
        let val = editor.get_value().unwrap();
        assert!(val.is::<String>());
        assert_eq!(val.downcast_ref::<String>().unwrap(), "de_DE");
    }

    #[test]
    fn get_value_when_empty() {
        let editor = LocaleEditor::new();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn various_locale_formats() {
        let mut editor = LocaleEditor::new();
        for locale in &["en", "en_US", "zh_CN", "ja_JP", "fr"] {
            editor.set_as_text(locale).unwrap();
            assert_eq!(editor.get_as_text(), Some(locale.to_string()));
        }
    }
}
