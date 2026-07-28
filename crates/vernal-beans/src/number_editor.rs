//! CustomNumberEditor — Spring 风格的数字编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CustomNumberEditor`。
//!
//! 将字符串转换为数字类型（i32/i64/f64 等）。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的数字编辑器。
///
/// 对应 Spring 的 `CustomNumberEditor`。
///
/// 支持以下数字类型：
/// - `i32` / `i64` / `u32` / `u64`
/// - `f32` / `f64`
///
/// 使用 `number_format` crate 解析本地化数字格式（如果启用）。
pub struct CustomNumberEditor {
    value: Option<f64>,
    allow_empty: bool,
}

impl CustomNumberEditor {
    /// 创建不允许空值的数字编辑器。
    pub fn new() -> Self {
        Self {
            value: None,
            allow_empty: false,
        }
    }

    /// 创建指定行为的数字编辑器。
    pub fn with_allow_empty(allow_empty: bool) -> Self {
        Self {
            value: None,
            allow_empty,
        }
    }
}

impl Default for CustomNumberEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for CustomNumberEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<f64>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            if self.allow_empty {
                self.value = None;
                return Ok(());
            }
            return Err("Empty string not allowed for number conversion".into());
        }
        // 尝试解析为 f64（支持整数和浮点数）
        let parsed = trimmed
            .parse::<f64>()
            .map_err(|e| format!("Cannot convert '{}' to number: {}", text, e))?;
        self.value = Some(parsed);
        Ok(())
    }

    fn get_as_text(&self) -> Option<String> {
        self.value.map(|v| {
            // 如果是整数值，不显示小数点
            if v.fract() == 0.0 && v.abs() < i64::MAX as f64 {
                (v as i64).to_string()
            } else {
                v.to_string()
            }
        })
    }

    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(v) = value.downcast_ref::<f64>() {
            self.value = Some(*v);
        } else if let Some(v) = value.downcast_ref::<i32>() {
            self.value = Some(*v as f64);
        } else if let Some(v) = value.downcast_ref::<i64>() {
            self.value = Some(*v as f64);
        } else if let Some(v) = value.downcast_ref::<u32>() {
            self.value = Some(*v as f64);
        } else if let Some(v) = value.downcast_ref::<u64>() {
            self.value = Some(*v as f64);
        } else if let Some(v) = value.downcast_ref::<f32>() {
            self.value = Some(*v as f64);
        }
    }

    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_ref().map(|v| v as &dyn Any)
    }

    fn get_value_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<f64>()
    }
}
