//! ResourceBundleEditor — Spring 风格的 ResourceBundle 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.ResourceBundleEditor`。
//! 将字符串转换为 ResourceBundle 标识（在 Rust 中用 String 表示）。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 ResourceBundle 编辑器。
pub struct ResourceBundleEditor {
    value: Option<String>,
}

impl ResourceBundleEditor {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for ResourceBundleEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for ResourceBundleEditor {
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
