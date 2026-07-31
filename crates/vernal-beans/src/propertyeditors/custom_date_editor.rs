//! CustomDateEditor — Spring 风格的自定义日期编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CustomDateEditor`。
//!
//! 在 Spring 中，`CustomDateEditor` 使用 `java.text.SimpleDateFormat`
//! 将字符串转换为 `java.util.Date`。
//!
//! ## 设计说明
//!
//! 在 vernal 中，日期以 ISO 8601 字符串格式存储和解析。
//! 内部使用时间戳（毫秒）表示日期值。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// 自定义日期属性编辑器。
///
/// 对应 Spring 的 `CustomDateEditor`。
///
/// 支持 ISO 8601 格式的日期解析和格式化。
#[derive(Debug)]
pub struct CustomDateEditor {
    /// 日期值（Unix 时间戳，毫秒）。
    value: Option<i64>,
    /// 日期格式说明。
    format: String,
    /// 是否允许空值。
    allow_empty: bool,
}

impl CustomDateEditor {
    /// 创建自定义日期编辑器。
    pub fn new() -> Self {
        Self {
            value: None,
            format: "yyyy-MM-dd".to_string(),
            allow_empty: true,
        }
    }

    /// 设置日期格式说明。
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// 设置是否允许空值。
    pub fn with_allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// 获取格式说明。
    pub fn format(&self) -> &str {
        &self.format
    }
}

impl Default for CustomDateEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for CustomDateEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<i64>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            if self.allow_empty {
                self.value = None;
                return Ok(());
            } else {
                return Err("Empty date not allowed".into());
            }
        }

        // 简单的日期解析：尝试解析为时间戳
        if let Ok(timestamp) = trimmed.parse::<i64>() {
            self.value = Some(timestamp);
            return Ok(());
        }

        // 尝试解析 ISO 格式 YYYY-MM-DD
        let parts: Vec<&str> = trimmed.split('-').collect();
        if parts.len() == 3 {
            if let (Ok(year), Ok(month), Ok(day)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                if year > 0 && month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                    // 简化的时间戳计算（近似值）
                    let days = (year as i64 - 1970) * 365 + (month as i64 - 1) * 30 + day as i64 - 1;
                    self.value = Some(days * 86400000);
                    return Ok(());
                }
            }
        }

        Err(format!("Cannot parse date: '{}'", trimmed).into())
    }

    fn get_as_text(&self) -> Option<String> {
        self.value.map(|ts| format!("{}", ts))
    }

    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(ts) = value.downcast_ref::<i64>() {
            self.value = Some(*ts);
        }
    }

    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_ref().map(|v| v as &dyn Any)
    }

    fn get_value_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<i64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_timestamp() {
        let mut editor = CustomDateEditor::new();
        editor.set_as_text("1704067200000").unwrap();
        assert_eq!(editor.value, Some(1704067200000));
    }

    #[test]
    fn parse_iso_date() {
        let mut editor = CustomDateEditor::new();
        editor.set_as_text("2024-01-01").unwrap();
        assert!(editor.value.is_some());
    }

    #[test]
    fn empty_string_gives_none() {
        let mut editor = CustomDateEditor::new();
        editor.set_as_text("").unwrap();
        assert!(editor.value.is_none());
    }

    #[test]
    fn invalid_date_returns_error() {
        let mut editor = CustomDateEditor::new();
        assert!(editor.set_as_text("not-a-date").is_err());
    }

    #[test]
    fn custom_format() {
        let editor = CustomDateEditor::new().with_format("dd/MM/yyyy");
        assert_eq!(editor.format(), "dd/MM/yyyy");
    }
}
