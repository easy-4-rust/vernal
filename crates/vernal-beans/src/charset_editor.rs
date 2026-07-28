//! CharsetEditor — Spring 风格的 Charset 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CharsetEditor`。
//! 将字符串转换为字符集名称（用 String 表示）。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 Charset 编辑器。
pub struct CharsetEditor {
    value: Option<String>,
}

impl CharsetEditor {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for CharsetEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for CharsetEditor {
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
