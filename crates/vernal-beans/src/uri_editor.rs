//! URIEditor — Spring 风格的 URI 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.URIEditor`。
//!
//! 将字符串转换为 `std::net::Uri`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 URI 编辑器。
///
/// 对应 Spring 的 `URIEditor`。
pub struct URIEditor {
    value: Option<String>,
}

impl URIEditor {
    /// 创建 URI 编辑器。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for URIEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for URIEditor {
    fn target_type(&self) -> std::any::TypeId {
        // URI 在 Rust 中没有统一类型，用 String 表示
        std::any::TypeId::of::<String>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.value = None;
            return Ok(());
        }
        // 验证 URI 格式（简单验证）
        if !trimmed.contains(':') && !trimmed.starts_with('/') {
            return Err(format!("Invalid URI: {}", text).into());
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
