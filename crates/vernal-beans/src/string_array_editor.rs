//! StringArrayPropertyEditor — Spring 风格的字符串数组编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.StringArrayPropertyEditor`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的字符串数组编辑器。
pub struct StringArrayPropertyEditor {
    value: Option<String>,
}

impl StringArrayPropertyEditor {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for StringArrayPropertyEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for StringArrayPropertyEditor {
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
