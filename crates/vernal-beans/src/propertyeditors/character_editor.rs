//! CharacterEditor — Spring 风格的 Character 属性编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CharacterEditor`。
//!
//! 将字符串转换为 `char` 值。如果字符串长度为 1，直接取第一个字符；
//! 如果为空字符串，设置为 `None`。
//!
//! ## 设计说明
//!
//! 在 Rust 中，`char` 是 4 字节 Unicode 标量值。
//! `CharacterEditor` 将字符串的第一个字符作为 `char` 值。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Character 属性编辑器。
///
/// 对应 Spring 的 `CharacterEditor`。
///
/// 将字符串转换为 `char` 值。
#[derive(Debug, Default)]
pub struct CharacterEditor {
    value: Option<char>,
}

impl CharacterEditor {
    /// 创建 Character 属性编辑器。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl PropertyEditor for CharacterEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<char>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if text.is_empty() {
            self.value = None;
        } else {
            self.value = text.chars().next();
        }
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
    fn set_single_char() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("A").unwrap();
        assert_eq!(editor.value, Some('A'));
        assert_eq!(editor.get_as_text(), Some("A".to_string()));
    }

    #[test]
    fn set_empty_string_gives_none() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("").unwrap();
        assert!(editor.value.is_none());
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_multibyte_char() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("中").unwrap();
        assert_eq!(editor.value, Some('中'));
    }

    #[test]
    fn set_value_directly() {
        let mut editor = CharacterEditor::new();
        editor.set_value(Arc::new('Z'));
        assert_eq!(editor.get_as_text(), Some("Z".to_string()));
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn default_creates_empty_editor() {
        let editor = CharacterEditor::default();
        assert!(editor.value.is_none());
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn target_type_is_char() {
        let editor = CharacterEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<char>());
    }

    #[test]
    fn get_value_type_is_char() {
        let editor = CharacterEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<char>());
    }

    #[test]
    fn get_value_returns_none_initially() {
        let editor = CharacterEditor::new();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn get_value_returns_some_after_set() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("X").unwrap();
        let val = editor.get_value().unwrap();
        assert_eq!(*val.downcast_ref::<char>().unwrap(), 'X');
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = CharacterEditor::new();
        editor.set_value(Arc::new("not a char".to_string()));
        // Should remain None since downcast_ref::<char> fails
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_as_text_unicode_emoji() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("\u{1F600}").unwrap();
        assert_eq!(editor.value, Some('\u{1F600}'));
    }

    #[test]
    fn set_as_text_first_char_of_multi() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("ABC").unwrap();
        assert_eq!(editor.value, Some('A'));
        assert_eq!(editor.get_as_text(), Some("A".to_string()));
    }

    #[test]
    fn set_as_text_digit() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("5").unwrap();
        assert_eq!(editor.value, Some('5'));
    }

    #[test]
    fn set_as_text_space() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text(" ").unwrap();
        assert_eq!(editor.value, Some(' '));
    }

    #[test]
    fn set_value_then_set_as_text_overrides() {
        let mut editor = CharacterEditor::new();
        editor.set_value(Arc::new('A'));
        assert_eq!(editor.get_as_text(), Some("A".to_string()));
        editor.set_as_text("B").unwrap();
        assert_eq!(editor.get_as_text(), Some("B".to_string()));
    }

    #[test]
    fn set_as_text_empty_then_nonempty() {
        let mut editor = CharacterEditor::new();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
        editor.set_as_text("Z").unwrap();
        assert_eq!(editor.get_as_text(), Some("Z".to_string()));
    }
}
