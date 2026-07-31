//! CustomBooleanEditor — Spring 风格的自定义 Boolean 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CustomBooleanEditor`。
//!
//! 比标准 `BooleanEditor` 更灵活，支持自定义的 true/false 字符串。
//! 例如可以配置 "yes/no"、"on/off"、"1/0" 等。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`CustomBooleanEditor` 支持配置自定义的 true/false 值列表。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// 自定义 Boolean 属性编辑器。
///
/// 对应 Spring 的 `CustomBooleanEditor`。
///
/// 支持自定义 true/false 字符串的布尔编辑器。
#[derive(Debug)]
pub struct CustomBooleanEditor {
    value: Option<bool>,
    true_strings: Vec<String>,
    false_strings: Vec<String>,
    allow_empty: bool,
}

impl CustomBooleanEditor {
    /// 创建自定义 Boolean 编辑器（使用默认 true/false 字符串）。
    pub fn new() -> Self {
        Self {
            value: None,
            true_strings: vec!["true".to_string(), "on".to_string(), "yes".to_string(), "1".to_string()],
            false_strings: vec!["false".to_string(), "off".to_string(), "no".to_string(), "0".to_string()],
            allow_empty: true,
        }
    }

    /// 设置自定义 true 字符串。
    pub fn with_true_strings(mut self, strings: Vec<String>) -> Self {
        self.true_strings = strings;
        self
    }

    /// 设置自定义 false 字符串。
    pub fn with_false_strings(mut self, strings: Vec<String>) -> Self {
        self.false_strings = strings;
        self
    }

    /// 设置是否允许空值。
    pub fn with_allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
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
        let trimmed = text.trim();
        if trimmed.is_empty() {
            if self.allow_empty {
                self.value = None;
                return Ok(());
            } else {
                return Err("Empty value not allowed".into());
            }
        }

        let lower = trimmed.to_lowercase();
        if self.true_strings.iter().any(|s| s.to_lowercase() == lower) {
            self.value = Some(true);
            Ok(())
        } else if self.false_strings.iter().any(|s| s.to_lowercase() == lower) {
            self.value = Some(false);
            Ok(())
        } else {
            Err(format!("Invalid boolean value: '{}'", trimmed).into())
        }
    }

    fn get_as_text(&self) -> Option<String> {
        self.value.map(|v| if v { "true".to_string() } else { "false".to_string() })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_true_values() {
        let mut editor = CustomBooleanEditor::new();
        for val in &["true", "TRUE", "on", "yes", "1"] {
            editor.set_as_text(val).unwrap();
            assert_eq!(editor.value, Some(true), "Failed for '{}'", val);
        }
    }

    #[test]
    fn standard_false_values() {
        let mut editor = CustomBooleanEditor::new();
        for val in &["false", "FALSE", "off", "no", "0"] {
            editor.set_as_text(val).unwrap();
            assert_eq!(editor.value, Some(false), "Failed for '{}'", val);
        }
    }

    #[test]
    fn invalid_value_returns_error() {
        let mut editor = CustomBooleanEditor::new();
        assert!(editor.set_as_text("maybe").is_err());
    }

    #[test]
    fn empty_value_allowed() {
        let mut editor = CustomBooleanEditor::new();
        editor.set_as_text("").unwrap();
        assert!(editor.value.is_none());
    }

    #[test]
    fn empty_value_not_allowed() {
        let mut editor = CustomBooleanEditor::new().with_allow_empty(false);
        assert!(editor.set_as_text("").is_err());
    }

    #[test]
    fn custom_true_false_strings() {
        let mut editor = CustomBooleanEditor::new()
            .with_true_strings(vec!["yep".to_string()])
            .with_false_strings(vec!["nope".to_string()]);
        editor.set_as_text("yep").unwrap();
        assert_eq!(editor.value, Some(true));
        editor.set_as_text("nope").unwrap();
        assert_eq!(editor.value, Some(false));
    }

    #[test]
    fn target_type_is_bool() {
        let editor = CustomBooleanEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<bool>());
    }

    #[test]
    fn get_value_type_is_bool() {
        let editor = CustomBooleanEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<bool>());
    }

    #[test]
    fn get_as_text_true() {
        let mut editor = CustomBooleanEditor::new();
        editor.set_as_text("true").unwrap();
        assert_eq!(editor.get_as_text(), Some("true".to_string()));
    }

    #[test]
    fn get_as_text_false() {
        let mut editor = CustomBooleanEditor::new();
        editor.set_as_text("false").unwrap();
        assert_eq!(editor.get_as_text(), Some("false".to_string()));
    }

    #[test]
    fn get_as_text_none_when_unset() {
        let editor = CustomBooleanEditor::new();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_value_with_bool() {
        let mut editor = CustomBooleanEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(true);
        editor.set_value(val);
        assert_eq!(editor.value, Some(true));
    }

    #[test]
    fn set_value_with_false() {
        let mut editor = CustomBooleanEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(false);
        editor.set_value(val);
        assert_eq!(editor.value, Some(false));
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = CustomBooleanEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new("not_a_bool".to_string());
        editor.set_value(val);
        assert!(editor.value.is_none());
    }

    #[test]
    fn get_value_returns_ref() {
        let mut editor = CustomBooleanEditor::new();
        editor.set_as_text("true").unwrap();
        let val = editor.get_value().unwrap();
        assert!(val.is::<bool>());
        assert_eq!(val.downcast_ref::<bool>().unwrap(), &true);
    }

    #[test]
    fn get_value_none_when_unset() {
        let editor = CustomBooleanEditor::new();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn whitespace_trimmed() {
        let mut editor = CustomBooleanEditor::new();
        editor.set_as_text("  true  ").unwrap();
        assert_eq!(editor.value, Some(true));
    }

    #[test]
    fn case_insensitive() {
        let mut editor = CustomBooleanEditor::new();
        editor.set_as_text("TRUE").unwrap();
        assert_eq!(editor.value, Some(true));
        editor.set_as_text("False").unwrap();
        assert_eq!(editor.value, Some(false));
        editor.set_as_text("YES").unwrap();
        assert_eq!(editor.value, Some(true));
        editor.set_as_text("NO").unwrap();
        assert_eq!(editor.value, Some(false));
    }

    #[test]
    fn default_trait() {
        let editor = CustomBooleanEditor::default();
        assert!(editor.value.is_none());
    }

    #[test]
    fn custom_strings_case_insensitive() {
        let mut editor = CustomBooleanEditor::new()
            .with_true_strings(vec!["YEP".to_string()])
            .with_false_strings(vec!["NOPE".to_string()]);
        editor.set_as_text("yep").unwrap();
        assert_eq!(editor.value, Some(true));
        editor.set_as_text("NOPE").unwrap();
        assert_eq!(editor.value, Some(false));
    }

    #[test]
    fn invalid_custom_string() {
        let mut editor = CustomBooleanEditor::new()
            .with_true_strings(vec!["affirmative".to_string()])
            .with_false_strings(vec!["negative".to_string()]);
        assert!(editor.set_as_text("yes").is_err());
    }
}
