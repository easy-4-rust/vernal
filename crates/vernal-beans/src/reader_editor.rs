//! ReaderEditor — Spring 风格的 Reader 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.ReaderEditor`。
//! 将字符串转换为 Reader 标识（用 String 表示）。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 Reader 编辑器。
pub struct ReaderEditor {
    value: Option<String>,
}

impl ReaderEditor {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for ReaderEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for ReaderEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.value = Some(text.to_string());
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
