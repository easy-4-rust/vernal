//! StringArrayPropertyEditor — Spring 风格的字符串数组属性编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.StringArrayPropertyEditor`。
//!
//! 在 Spring 中，`StringArrayPropertyEditor` 将逗号分隔的字符串
//! 转换为字符串数组。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`StringArrayPropertyEditor` 管理 `Vec<String>` 值，
//! 支持逗号分隔和自定义分隔符。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// 字符串数组属性编辑器。
///
/// 对应 Spring 的 `StringArrayPropertyEditor`。
///
/// 将逗号分隔的字符串转换为字符串数组。
#[derive(Debug)]
pub struct StringArrayPropertyEditor {
    value: Vec<String>,
    separator: String,
    trim_items: bool,
    allow_empty: bool,
}

impl StringArrayPropertyEditor {
    /// 创建字符串数组属性编辑器（默认逗号分隔）。
    pub fn new() -> Self {
        Self {
            value: Vec::new(),
            separator: ",".to_string(),
            trim_items: true,
            allow_empty: true,
        }
    }

    /// 设置分隔符。
    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// 设置是否裁剪元素。
    pub fn with_trim(mut self, trim: bool) -> Self {
        self.trim_items = trim;
        self
    }

    /// 设置是否允许空值。
    pub fn with_allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// 获取元素数量。
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl Default for StringArrayPropertyEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for StringArrayPropertyEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Vec<String>>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if text.is_empty() {
            if self.allow_empty {
                self.value.clear();
                return Ok(());
            } else {
                return Err("Empty value not allowed".into());
            }
        }

        self.value = text
            .split(&self.separator)
            .map(|s| {
                if self.trim_items {
                    s.trim().to_string()
                } else {
                    s.to_string()
                }
            })
            .collect();

        Ok(())
    }

    fn get_as_text(&self) -> Option<String> {
        Some(self.value.join(&self.separator))
    }

    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(v) = value.downcast_ref::<Vec<String>>() {
            self.value = v.clone();
        }
    }

    fn get_value(&self) -> Option<&dyn Any> {
        Some(&self.value)
    }

    fn get_value_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Vec<String>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_comma_separated() {
        let mut editor = StringArrayPropertyEditor::new();
        editor.set_as_text("a, b, c").unwrap();
        assert_eq!(editor.value, vec!["a", "b", "c"]);
    }

    #[test]
    fn custom_separator() {
        let mut editor = StringArrayPropertyEditor::new().with_separator("|");
        editor.set_as_text("x|y|z").unwrap();
        assert_eq!(editor.value, vec!["x", "y", "z"]);
    }

    #[test]
    fn no_trim() {
        let mut editor = StringArrayPropertyEditor::new().with_trim(false);
        editor.set_as_text(" a , b ").unwrap();
        assert_eq!(editor.value, vec![" a ", " b "]);
    }

    #[test]
    fn empty_string_clears() {
        let mut editor = StringArrayPropertyEditor::new();
        editor.set_as_text("a,b").unwrap();
        assert_eq!(editor.len(), 2);
        editor.set_as_text("").unwrap();
        assert!(editor.is_empty());
    }

    #[test]
    fn get_as_text_roundtrip() {
        let mut editor = StringArrayPropertyEditor::new();
        editor.set_as_text("hello, world").unwrap();
        // 分隔符是 ","，trim 后元素为 ["hello", "world"]，join 结果为 "hello,world"
        assert_eq!(editor.get_as_text(), Some("hello,world".to_string()));
    }

    #[test]
    fn target_type_is_vec_string() {
        let editor = StringArrayPropertyEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<Vec<String>>());
    }

    #[test]
    fn get_value_type_is_vec_string() {
        let editor = StringArrayPropertyEditor::new();
        assert_eq!(
            editor.get_value_type(),
            std::any::TypeId::of::<Vec<String>>()
        );
    }

    #[test]
    fn set_value_with_vec_string() {
        let mut editor = StringArrayPropertyEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> =
            Arc::new(vec!["a".to_string(), "b".to_string()]);
        editor.set_value(val);
        assert_eq!(editor.len(), 2);
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = StringArrayPropertyEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.is_empty());
    }

    #[test]
    fn get_value_returns_ref() {
        let mut editor = StringArrayPropertyEditor::new();
        editor.set_as_text("x,y").unwrap();
        let val = editor.get_value().unwrap();
        assert!(val.is::<Vec<String>>());
    }

    #[test]
    fn empty_not_allowed() {
        let mut editor = StringArrayPropertyEditor::new().with_allow_empty(false);
        assert!(editor.set_as_text("").is_err());
    }

    #[test]
    fn single_element() {
        let mut editor = StringArrayPropertyEditor::new();
        editor.set_as_text("only_one").unwrap();
        assert_eq!(editor.len(), 1);
        assert_eq!(editor.value, vec!["only_one"]);
    }

    #[test]
    fn default_trait() {
        let editor = StringArrayPropertyEditor::default();
        assert!(editor.is_empty());
    }

    #[test]
    fn pipe_separator() {
        let mut editor = StringArrayPropertyEditor::new().with_separator("||");
        editor.set_as_text("a||b||c").unwrap();
        assert_eq!(editor.value, vec!["a", "b", "c"]);
    }

    #[test]
    fn get_as_text_with_custom_separator() {
        let mut editor = StringArrayPropertyEditor::new().with_separator("|");
        editor.set_as_text("x|y|z").unwrap();
        assert_eq!(editor.get_as_text(), Some("x|y|z".to_string()));
    }
}
