//! CustomBooleanEditor — Spring 风格的布尔编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CustomBooleanEditor`。
//!
//! 将字符串 "true"/"false"/"yes"/"no"/"1"/"0" 转换为 bool。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的布尔编辑器。
///
/// 对应 Spring 的 `CustomBooleanEditor`。
///
/// 支持以下字符串表示：
/// - `true` / `false`
/// - `yes` / `no`
/// - `1` / `0`
/// - 大小写不敏感
pub struct CustomBooleanEditor {
    value: Option<bool>,
    allow_empty: bool,
}

impl CustomBooleanEditor {
    /// 创建不允许空值的布尔编辑器。
    pub fn new() -> Self {
        Self {
            value: None,
            allow_empty: false,
        }
    }

    /// 创建指定行为的布尔编辑器。
    pub fn with_allow_empty(allow_empty: bool) -> Self {
        Self {
            value: None,
            allow_empty,
        }
    }
}

impl Default for CustomBooleanEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for CustomBooleanEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<bool>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim().to_lowercase();
        if trimmed.is_empty() {
            if self.allow_empty {
                self.value = None;
                return Ok(());
            }
            return Err("Empty string not allowed for boolean conversion".into());
        }
        match trimmed.as_str() {
            "true" | "yes" | "1" => {
                self.value = Some(true);
                Ok(())
            }
            "false" | "no" | "0" => {
                self.value = Some(false);
                Ok(())
            }
            _ => Err(format!("Cannot convert '{}' to boolean", text).into()),
        }
    }

    fn get_as_text(&self) -> Option<String> {
        self.value.map(|v| v.to_string())
    }

    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(b) = value.downcast_ref::<bool>() {
            self.value = Some(*b);
        }
    }

    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_ref().map(|v| v as &dyn Any)
    }

    fn get_value_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<bool>()
    }
}
