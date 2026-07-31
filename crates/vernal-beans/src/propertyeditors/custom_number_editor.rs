//! CustomNumberEditor — Spring 风格的自定义数字编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CustomNumberEditor`。
//!
//! 在 Spring 中，`CustomNumberEditor` 将字符串转换为指定的数字类型
//! （Integer, Long, Float, Double 等）。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`CustomNumberEditor` 支持将字符串解析为 i64 或 f64，
//! 并根据目标类型进行转换。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// 数字类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberType {
    /// 64 位整数。
    I64,
    /// 64 位浮点数。
    F64,
}

/// 自定义数字属性编辑器。
///
/// 对应 Spring 的 `CustomNumberEditor`。
///
/// 将字符串解析为指定类型的数字。
#[derive(Debug)]
pub struct CustomNumberEditor {
    number_type: NumberType,
    value: Option<f64>,
    allow_empty: bool,
}

impl CustomNumberEditor {
    /// 创建自定义数字编辑器。
    pub fn new(number_type: NumberType) -> Self {
        Self {
            number_type,
            value: None,
            allow_empty: true,
        }
    }

    /// 设置是否允许空值。
    pub fn with_allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// 获取数字类型。
    pub fn number_type(&self) -> NumberType {
        self.number_type
    }

    /// 获取整数值（如果是 i64 类型）。
    pub fn as_i64(&self) -> Option<i64> {
        self.value.map(|v| v as i64)
    }

    /// 获取浮点值。
    pub fn as_f64(&self) -> Option<f64> {
        self.value
    }
}

impl PropertyEditor for CustomNumberEditor {
    fn target_type(&self) -> std::any::TypeId {
        match self.number_type {
            NumberType::I64 => std::any::TypeId::of::<i64>(),
            NumberType::F64 => std::any::TypeId::of::<f64>(),
        }
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            if self.allow_empty {
                self.value = None;
                return Ok(());
            } else {
                return Err("Empty number not allowed".into());
            }
        }

        match self.number_type {
            NumberType::I64 => {
                let v: i64 = trimmed.parse()
                    .map_err(|_| format!("Cannot parse '{}' as i64", trimmed))?;
                self.value = Some(v as f64);
            }
            NumberType::F64 => {
                let v: f64 = trimmed.parse()
                    .map_err(|_| format!("Cannot parse '{}' as f64", trimmed))?;
                self.value = Some(v);
            }
        }
        Ok(())
    }

    fn get_as_text(&self) -> Option<String> {
        self.value.map(|v| match self.number_type {
            NumberType::I64 => format!("{}", v as i64),
            NumberType::F64 => format!("{}", v),
        })
    }

    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(v) = value.downcast_ref::<i64>() {
            self.value = Some(*v as f64);
        } else if let Some(v) = value.downcast_ref::<f64>() {
            self.value = Some(*v);
        }
    }

    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_ref().map(|v| v as &dyn Any)
    }

    fn get_value_type(&self) -> std::any::TypeId {
        match self.number_type {
            NumberType::I64 => std::any::TypeId::of::<i64>(),
            NumberType::F64 => std::any::TypeId::of::<f64>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_i64() {
        let mut editor = CustomNumberEditor::new(NumberType::I64);
        editor.set_as_text("42").unwrap();
        assert_eq!(editor.as_i64(), Some(42));
    }

    #[test]
    fn parse_f64() {
        let mut editor = CustomNumberEditor::new(NumberType::F64);
        editor.set_as_text("3.14").unwrap();
        assert!((editor.as_f64().unwrap() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn invalid_number_returns_error() {
        let mut editor = CustomNumberEditor::new(NumberType::I64);
        assert!(editor.set_as_text("abc").is_err());
    }

    #[test]
    fn empty_string_gives_none() {
        let mut editor = CustomNumberEditor::new(NumberType::I64);
        editor.set_as_text("42").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.as_i64().is_none());
    }

    #[test]
    fn get_as_text_format() {
        let mut editor = CustomNumberEditor::new(NumberType::I64);
        editor.set_as_text("100").unwrap();
        assert_eq!(editor.get_as_text(), Some("100".to_string()));
    }
}
