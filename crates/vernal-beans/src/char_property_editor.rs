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
