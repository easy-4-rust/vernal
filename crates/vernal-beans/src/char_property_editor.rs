//! CharacterEditor — Spring 风格的字符编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CharacterEditor`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的字符编辑器。
pub struct CharacterEditor {
    value: Option<char>,
}

impl CharacterEditor {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for CharacterEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for CharacterEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<char>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.value = None;
            return Ok(());
        }
        let ch = trimmed.chars().next().ok_or("Empty string")?;
        self.value = Some(ch);
        Ok(())
    }
    fn get_as_text(&self) -> Option<String> {
        self.value.map(|c| c.to_string())
    }
    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(c) = value.downcast_ref::<char>() {
            self.value = Some(*c);
        }
    }
    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_ref().map(|v| v as &dyn Any)
    }
    fn get_value_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<char>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_editor_has_no_value() {
        let editor = CharacterEditor::new();
        assert!(editor.value.is_none());
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn default_trait() {
        let editor = CharacterEditor::default();
        assert!(editor.value.is_none());
    }

    #[test]
    fn set_as_text_single_char() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("A").unwrap();
        assert_eq!(editor.value, Some('A'));
        assert_eq!(editor.get_as_text(), Some("A".to_string()));
    }

    #[test]
    fn set_as_text_multiple_chars_takes_first() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("ABC").unwrap();
        assert_eq!(editor.value, Some('A'));
    }

    #[test]
    fn set_as_text_trims_whitespace() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("  Z  ").unwrap();
        assert_eq!(editor.value, Some('Z'));
    }

    #[test]
    fn set_as_text_empty_clears_value() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("X").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.value.is_none());
    }

    #[test]
    fn set_as_text_whitespace_only_clears() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("X").unwrap();
        editor.set_as_text("   ").unwrap();
        assert!(editor.value.is_none());
    }

    #[test]
    fn set_and_get_value() {
        let mut editor = CharacterEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new('B');
        editor.set_value(val);
        let retrieved = editor.get_value().unwrap();
        assert_eq!(*retrieved.downcast_ref::<char>().unwrap(), 'B');
    }

    #[test]
    fn set_value_wrong_type_ignored() {
        let mut editor = CharacterEditor::new();
        let val: Arc<dyn Any + Send + Sync> = Arc::new("not a char".to_string());
        editor.set_value(val);
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn target_and_value_types() {
        let editor = CharacterEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<char>());
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<char>());
    }
}
