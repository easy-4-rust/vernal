//! UUIDEditor — Spring 风格的 UUID 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.UUIDEditor`。
//!
//! 将字符串转换为 `std::uuid::Uuid`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 UUID 编辑器。
///
/// 对应 Spring 的 `UUIDEditor`。
pub struct UUIDEditor {
    value: Option<String>,
}

impl UUIDEditor {
    /// 创建 UUID 编辑器。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for UUIDEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for UUIDEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.value = None;
            return Ok(());
        }
        // 验证 UUID 格式（8-4-4-4-12）
        let parts: Vec<&str> = trimmed.split('-').collect();
        if parts.len() != 5 {
            return Err(format!("Invalid UUID format: {}", text).into());
        }
        if parts[0].len() != 8
            || parts[1].len() != 4
            || parts[2].len() != 4
            || parts[3].len() != 4
            || parts[4].len() != 12
        {
            return Err(format!("Invalid UUID format: {}", text).into());
        }
        // 验证每个字符都是十六进制
        for part in &parts {
            if !part.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(format!("Invalid UUID format: {}", text).into());
            }
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
