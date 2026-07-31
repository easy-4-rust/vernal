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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_editor_has_no_value() {
        let editor = CustomNumberEditor::new();
        assert!(editor.get_value().is_none());
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn default_trait_creates_same_as_new() {
        let editor = CustomNumberEditor::default();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn with_allow_empty_true() {
        let mut editor = CustomNumberEditor::with_allow_empty(true);
        editor.set_as_text("").unwrap();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn with_allow_empty_false_rejects_empty() {
        let mut editor = CustomNumberEditor::with_allow_empty(false);
        let result = editor.set_as_text("");
        assert!(result.is_err());
    }

    #[test]
    fn set_as_text_integer() {
        let mut editor = CustomNumberEditor::new();
        editor.set_as_text("42").unwrap();
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert_eq!(*val, 42.0);
    }

    #[test]
    fn set_as_text_float() {
        let mut editor = CustomNumberEditor::new();
        editor.set_as_text("3.14").unwrap();
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn set_as_text_negative() {
        let mut editor = CustomNumberEditor::new();
        editor.set_as_text("-100").unwrap();
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert_eq!(*val, -100.0);
    }

    #[test]
    fn set_as_text_zero() {
        let mut editor = CustomNumberEditor::new();
        editor.set_as_text("0").unwrap();
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert_eq!(*val, 0.0);
    }

    #[test]
    fn set_as_text_whitespace_trimmed() {
        let mut editor = CustomNumberEditor::new();
        editor.set_as_text("  42  ").unwrap();
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert_eq!(*val, 42.0);
    }

    #[test]
    fn set_as_text_invalid_returns_error() {
        let mut editor = CustomNumberEditor::new();
        let result = editor.set_as_text("not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn get_as_text_integer_format() {
        let mut editor = CustomNumberEditor::new();
        editor.set_as_text("42").unwrap();
        assert_eq!(editor.get_as_text(), Some("42".to_string()));
    }

    #[test]
    fn get_as_text_float_format() {
        let mut editor = CustomNumberEditor::new();
        editor.set_as_text("3.14").unwrap();
        let text = editor.get_as_text().unwrap();
        assert!(text.contains("3.14"));
    }

    #[test]
    fn get_as_text_negative_integer() {
        let mut editor = CustomNumberEditor::new();
        editor.set_as_text("-5").unwrap();
        assert_eq!(editor.get_as_text(), Some("-5".to_string()));
    }

    #[test]
    fn set_value_from_f64() {
        let mut editor = CustomNumberEditor::new();
        editor.set_value(Arc::new(3.14f64));
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn set_value_from_i32() {
        let mut editor = CustomNumberEditor::new();
        editor.set_value(Arc::new(42i32));
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert_eq!(*val, 42.0);
    }

    #[test]
    fn set_value_from_i64() {
        let mut editor = CustomNumberEditor::new();
        editor.set_value(Arc::new(100i64));
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert_eq!(*val, 100.0);
    }

    #[test]
    fn set_value_from_u32() {
        let mut editor = CustomNumberEditor::new();
        editor.set_value(Arc::new(99u32));
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert_eq!(*val, 99.0);
    }

    #[test]
    fn set_value_from_u64() {
        let mut editor = CustomNumberEditor::new();
        editor.set_value(Arc::new(255u64));
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert_eq!(*val, 255.0);
    }

    #[test]
    fn set_value_from_f32() {
        let mut editor = CustomNumberEditor::new();
        editor.set_value(Arc::new(1.5f32));
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert!((val - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn set_value_from_unsupported_type_ignored() {
        let mut editor = CustomNumberEditor::new();
        editor.set_value(Arc::new("not a number".to_string()));
        // Unsupported types are silently ignored
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn target_type_is_f64() {
        let editor = CustomNumberEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<f64>());
    }

    #[test]
    fn get_value_type_is_f64() {
        let editor = CustomNumberEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<f64>());
    }

    #[test]
    fn set_as_text_large_number() {
        let mut editor = CustomNumberEditor::new();
        editor.set_as_text("999999999").unwrap();
        let val = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
        assert_eq!(*val, 999999999.0);
    }

    #[test]
    fn set_as_text_scientific_notation() {
        let mut editor = CustomNumberEditor::new();
        let result = editor.set_as_text("1e10");
        // f64::parse supports scientific notation
        assert!(result.is_ok());
    }
}
