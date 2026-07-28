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
