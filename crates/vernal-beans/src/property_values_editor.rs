//! PropertyValuesEditor — 对应 Spring `org.springframework.beans.PropertyValuesEditor`。
//!
//! 属性值编辑器。

use std::any::{Any, TypeId};
use std::sync::Arc;

use crate::property_editor::PropertyEditor;
use crate::property_values::{MutablePropertyValuesImpl, PropertyValues};

/// 属性值编辑器。
///
/// 对应 Java 类：`org.springframework.beans.PropertyValuesEditor`。
///
/// 将文本解析为 PropertyValues。
#[derive(Debug, Default)]
pub struct PropertyValuesEditor {
    value: Option<MutablePropertyValuesImpl>,
}

impl PropertyValuesEditor {
    /// 创建一个新的 PropertyValuesEditor。
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取属性值。
    pub fn property_values(&self) -> Option<&dyn PropertyValues> {
        self.value.as_ref().map(|v| v as &dyn PropertyValues)
    }
}

impl PropertyEditor for PropertyValuesEditor {
    fn target_type(&self) -> TypeId {
        TypeId::of::<MutablePropertyValuesImpl>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut pvs = MutablePropertyValuesImpl::new();
        // 简单的文本解析：按逗号分割 key=value 对
        for part in text.split(',') {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                pvs.add(crate::property_value::PropertyValue::new(
                    key.trim(),
                    std::sync::Arc::new(value.trim().to_string())
                        as std::sync::Arc<dyn std::any::Any + Send + Sync>,
                ));
            }
        }
        self.value = Some(pvs);
        Ok(())
    }

    fn get_as_text(&self) -> Option<String> {
        self.value.as_ref().map(|pvs| {
            pvs.property_values()
                .iter()
                .map(|pv| {
                    let value_str = pv
                        .value()
                        .downcast_ref::<String>()
                        .map(|s| s.as_str())
                        .unwrap_or("?");
                    format!("{}={}", pv.name(), value_str)
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
    }

    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(pvs) = value.downcast_ref::<MutablePropertyValuesImpl>() {
            self.value = Some(pvs.clone());
        }
    }

    fn get_value(&self) -> Option<&dyn Any> {
        self.value.as_ref().map(|v| v as &dyn Any)
    }

    fn get_value_type(&self) -> TypeId {
        TypeId::of::<MutablePropertyValuesImpl>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let editor = PropertyValuesEditor::new();
        assert!(editor.property_values().is_none());
    }

    #[test]
    fn test_set_as_text() {
        let mut editor = PropertyValuesEditor::new();
        editor.set_as_text("name=Alice, age=30").unwrap();
        let pvs = editor.property_values().unwrap();
        assert_eq!(pvs.len(), 2);
        assert!(pvs.contains("name"));
        assert!(pvs.contains("age"));
    }

    #[test]
    fn test_get_as_text() {
        let mut editor = PropertyValuesEditor::new();
        editor.set_as_text("name=Alice, age=30").unwrap();
        let text = editor.get_as_text().unwrap();
        assert!(text.contains("name"));
        assert!(text.contains("Alice"));
    }

    #[test]
    fn test_target_type() {
        let editor = PropertyValuesEditor::new();
        assert_eq!(
            editor.target_type(),
            TypeId::of::<MutablePropertyValuesImpl>()
        );
    }

    #[test]
    fn test_get_value_type() {
        let editor = PropertyValuesEditor::new();
        assert_eq!(
            editor.get_value_type(),
            TypeId::of::<MutablePropertyValuesImpl>()
        );
    }

    #[test]
    fn test_get_as_text_empty() {
        let editor = PropertyValuesEditor::new();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn test_get_value_empty() {
        let editor = PropertyValuesEditor::new();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn test_get_value_after_set() {
        let mut editor = PropertyValuesEditor::new();
        editor.set_as_text("key=value").unwrap();
        assert!(editor.get_value().is_some());
        assert!(
            editor
                .get_value()
                .unwrap()
                .is::<MutablePropertyValuesImpl>()
        );
    }

    #[test]
    fn test_set_as_text_single_pair() {
        let mut editor = PropertyValuesEditor::new();
        editor.set_as_text("host=localhost").unwrap();
        let pvs = editor.property_values().unwrap();
        assert_eq!(pvs.len(), 1);
        assert!(pvs.contains("host"));
    }

    #[test]
    fn test_set_as_text_empty_string() {
        let mut editor = PropertyValuesEditor::new();
        editor.set_as_text("").unwrap();
        let pvs = editor.property_values().unwrap();
        assert_eq!(pvs.len(), 0);
    }

    #[test]
    fn test_set_as_text_no_equals() {
        let mut editor = PropertyValuesEditor::new();
        editor.set_as_text("noequalssign").unwrap();
        let pvs = editor.property_values().unwrap();
        assert_eq!(pvs.len(), 0);
    }

    #[test]
    fn test_set_as_text_whitespace_trimmed() {
        let mut editor = PropertyValuesEditor::new();
        editor
            .set_as_text("  key = value  ,  key2 = value2  ")
            .unwrap();
        let pvs = editor.property_values().unwrap();
        assert_eq!(pvs.len(), 2);
    }

    #[test]
    fn test_set_value_with_mutable_property_values() {
        let mut editor = PropertyValuesEditor::new();
        let mut pvs = MutablePropertyValuesImpl::new();
        pvs.add(crate::property_value::PropertyValue::new(
            "test_key",
            Arc::new("test_value".to_string()) as Arc<dyn std::any::Any + Send + Sync>,
        ));
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(pvs);
        editor.set_value(val);
        assert!(editor.property_values().is_some());
        assert_eq!(editor.property_values().unwrap().len(), 1);
    }

    #[test]
    fn test_set_value_with_wrong_type_ignored() {
        let mut editor = PropertyValuesEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> =
            Arc::new("not_a_property_values".to_string());
        editor.set_value(val);
        assert!(editor.property_values().is_none());
    }

    #[test]
    fn test_default_trait() {
        let editor = PropertyValuesEditor::default();
        assert!(editor.property_values().is_none());
    }

    #[test]
    fn test_get_as_text_roundtrip() {
        let mut editor = PropertyValuesEditor::new();
        editor.set_as_text("name=Alice, age=30").unwrap();
        let text = editor.get_as_text().unwrap();
        // Should contain both key=value pairs
        assert!(text.contains("name=Alice"));
        assert!(text.contains("age=30"));
    }
}
