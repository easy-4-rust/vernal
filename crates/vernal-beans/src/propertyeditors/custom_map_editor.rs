//! CustomMapEditor — Spring 风格的自定义 Map 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CustomMapEditor`。
//!
//! 在 Spring 中，`CustomMapEditor` 将字符串或其他 Map 转换为指定类型的 Map。
//! 支持 JSON 格式的键值对。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`CustomMapEditor` 使用 `HashMap<String, String>` 存储映射，
//! 支持 JSON 格式和简单 key=value 格式解析。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// 自定义 Map 属性编辑器。
///
/// 对应 Spring 的 `CustomMapEditor`。
///
/// 管理字符串到字符串的映射值。
#[derive(Debug)]
pub struct CustomMapEditor {
    value: HashMap<String, String>,
    allow_empty: bool,
}

impl CustomMapEditor {
    /// 创建自定义 Map 编辑器。
    pub fn new() -> Self {
        Self {
            value: HashMap::new(),
            allow_empty: true,
        }
    }

    /// 设置是否允许空值。
    pub fn with_allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// 获取映射大小。
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl Default for CustomMapEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for CustomMapEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<HashMap<String, String>>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            if self.allow_empty {
                self.value.clear();
                return Ok(());
            } else {
                return Err("Empty map not allowed".into());
            }
        }

        // 尝试 JSON 解析
        if trimmed.starts_with('{') {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(trimmed) {
                self.value = map;
                return Ok(());
            }
        }

        // 解析 key=value,key2=value2 格式
        let mut map = HashMap::new();
        for pair in trimmed.split(',') {
            let pair = pair.trim();
            if let Some(eq_pos) = pair.find('=') {
                let key = pair[..eq_pos].trim().to_string();
                let val = pair[eq_pos + 1..].trim().to_string();
                map.insert(key, val);
            }
        }
        self.value = map;
        Ok(())
    }

    fn get_as_text(&self) -> Option<String> {
        if self.value.is_empty() {
            return Some("{}".to_string());
        }
        let entries: Vec<String> = self.value.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        Some(entries.join(", "))
    }

    fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        if let Some(m) = value.downcast_ref::<HashMap<String, String>>() {
            self.value = m.clone();
        }
    }

    fn get_value(&self) -> Option<&dyn Any> {
        Some(&self.value)
    }

    fn get_value_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<HashMap<String, String>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_value_pairs() {
        let mut editor = CustomMapEditor::new();
        editor.set_as_text("name=John, age=30").unwrap();
        assert_eq!(editor.value.get("name"), Some(&"John".to_string()));
        assert_eq!(editor.value.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn parse_json_format() {
        let mut editor = CustomMapEditor::new();
        editor.set_as_text(r#"{"key":"value","num":"42"}"#).unwrap();
        assert_eq!(editor.value.get("key"), Some(&"value".to_string()));
        assert_eq!(editor.value.get("num"), Some(&"42".to_string()));
    }

    #[test]
    fn empty_string_clears_map() {
        let mut editor = CustomMapEditor::new();
        editor.set_as_text("a=1").unwrap();
        assert_eq!(editor.len(), 1);
        editor.set_as_text("").unwrap();
        assert!(editor.is_empty());
    }

    #[test]
    fn get_as_text_format() {
        let mut editor = CustomMapEditor::new();
        editor.set_as_text("x=1, y=2").unwrap();
        let text = editor.get_as_text().unwrap();
        assert!(text.contains("x=1"));
        assert!(text.contains("y=2"));
    }
}
