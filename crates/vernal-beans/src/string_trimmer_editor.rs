//! StringTrimmerEditor — Spring 风格的字符串修剪编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.StringTrimmerEditor`。
//!
//! 将字符串值修剪空白字符，空字符串转为 `None`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的字符串修剪编辑器。
///
/// 对应 Spring 的 `StringTrimmerEditor`。
///
/// - 将输入字符串修剪首尾空白
/// - 如果修剪后为空字符串，返回 `None`（可配置）
/// - 支持 trim empty → null 转换
pub struct StringTrimmerEditor {
    /// 修剪后的值。
    value: Option<String>,
    /// 空字符串是否转为 None。
    empty_as_null: bool,
}

impl StringTrimmerEditor {
    /// 创建默认的 StringTrimmerEditor（空字符串转 None）。
    pub fn new() -> Self {
        Self {
            value: None,
            empty_as_null: true,
        }
    }

    /// 创建指定行为的 StringTrimmerEditor。
    pub fn with_empty_as_null(empty_as_null: bool) -> Self {
        Self {
            value: None,
            empty_as_null,
        }
    }
}

impl Default for StringTrimmerEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for StringTrimmerEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() && self.empty_as_null {
            self.value = None;
        } else {
            self.value = Some(trimmed.to_string());
        }
        Ok(())
    }

    fn get_as_text(&self) -> Option<String> {
        self.value.clone()
    }

    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(s) = value.downcast_ref::<String>() {
            let trimmed = s.trim();
            if trimmed.is_empty() && self.empty_as_null {
                self.value = None;
            } else {
                self.value = Some(trimmed.to_string());
            }
        }
    }

    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_ref().map(|v| v as &dyn Any)
    }

    fn get_value_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
}
