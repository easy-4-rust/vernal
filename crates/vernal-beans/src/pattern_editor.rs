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
