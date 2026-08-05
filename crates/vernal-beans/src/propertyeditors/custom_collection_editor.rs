//! CustomCollectionEditor — Spring 风格的自定义集合编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.CustomCollectionEditor`。
//!
//! 在 Spring 中，`CustomCollectionEditor` 将字符串数组或其他集合
//! 转换为指定类型的集合（List, Set 等）。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`CustomCollectionEditor` 管理字符串列表，
//! 支持从逗号分隔的字符串解析。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// 集合类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionType {
    /// 列表（允许重复）。
    List,
    /// 集合（去重）。
    Set,
}

/// 自定义集合属性编辑器。
///
/// 对应 Spring 的 `CustomCollectionEditor`。
///
/// 管理字符串集合值。
#[derive(Debug)]
pub struct CustomCollectionEditor {
    value: Vec<String>,
    collection_type: CollectionType,
    allow_empty: bool,
}

impl CustomCollectionEditor {
    /// 创建自定义集合编辑器。
    pub fn new(collection_type: CollectionType) -> Self {
        Self {
            value: Vec::new(),
            collection_type,
            allow_empty: true,
        }
    }

    /// 设置是否允许空值。
    pub fn with_allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// 获取集合类型。
    pub fn collection_type(&self) -> CollectionType {
        self.collection_type
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

impl PropertyEditor for CustomCollectionEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Vec<String>>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            if self.allow_empty {
                self.value.clear();
                return Ok(());
            } else {
                return Err("Empty collection not allowed".into());
            }
        }

        let items: Vec<String> = trimmed
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        self.value = match self.collection_type {
            CollectionType::List => items,
            CollectionType::Set => {
                let mut unique: Vec<String> = Vec::new();
                for item in items {
                    if !unique.contains(&item) {
                        unique.push(item);
                    }
                }
                unique
            }
        };

        Ok(())
    }

    fn get_as_text(&self) -> Option<String> {
        Some(self.value.join(", "))
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
    fn parse_comma_separated_list() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        editor.set_as_text("a, b, c").unwrap();
        assert_eq!(editor.value, vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_set_removes_duplicates() {
        let mut editor = CustomCollectionEditor::new(CollectionType::Set);
        editor.set_as_text("a, b, a, c, b").unwrap();
        assert_eq!(editor.value, vec!["a", "b", "c"]);
    }

    #[test]
    fn empty_string_clears_collection() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        editor.set_as_text("a, b").unwrap();
        assert_eq!(editor.len(), 2);
        editor.set_as_text("").unwrap();
        assert!(editor.is_empty());
    }

    #[test]
    fn get_as_text_format() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        editor.set_as_text("x, y, z").unwrap();
        assert_eq!(editor.get_as_text(), Some("x, y, z".to_string()));
    }

    #[test]
    fn empty_not_allowed() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List).with_allow_empty(false);
        assert!(editor.set_as_text("").is_err());
    }

    #[test]
    fn target_type_is_vec_string() {
        let editor = CustomCollectionEditor::new(CollectionType::List);
        assert_eq!(editor.target_type(), std::any::TypeId::of::<Vec<String>>());
    }

    #[test]
    fn get_value_type_is_vec_string() {
        let editor = CustomCollectionEditor::new(CollectionType::List);
        assert_eq!(
            editor.get_value_type(),
            std::any::TypeId::of::<Vec<String>>()
        );
    }

    #[test]
    fn set_value_with_vec_string() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        let val: Arc<dyn std::any::Any + Send + Sync> =
            Arc::new(vec!["a".to_string(), "b".to_string()]);
        editor.set_value(val);
        assert_eq!(editor.len(), 2);
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.is_empty());
    }

    #[test]
    fn get_value_returns_ref() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        editor.set_as_text("a,b").unwrap();
        let val = editor.get_value().unwrap();
        assert!(val.is::<Vec<String>>());
    }

    #[test]
    fn collection_type_list() {
        let editor = CustomCollectionEditor::new(CollectionType::List);
        assert_eq!(editor.collection_type(), CollectionType::List);
    }

    #[test]
    fn collection_type_set() {
        let editor = CustomCollectionEditor::new(CollectionType::Set);
        assert_eq!(editor.collection_type(), CollectionType::Set);
    }

    #[test]
    fn list_allows_duplicates() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        editor.set_as_text("a, b, a, c, b").unwrap();
        assert_eq!(editor.value, vec!["a", "b", "a", "c", "b"]);
    }

    #[test]
    fn set_removes_empty_items() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        editor.set_as_text("a, , b, , c").unwrap();
        assert_eq!(editor.value, vec!["a", "b", "c"]);
    }

    #[test]
    fn whitespace_trimmed() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        editor.set_as_text("  a  ,  b  ,  c  ").unwrap();
        assert_eq!(editor.value, vec!["a", "b", "c"]);
    }

    #[test]
    fn single_element() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        editor.set_as_text("only").unwrap();
        assert_eq!(editor.len(), 1);
        assert_eq!(editor.value, vec!["only"]);
    }

    #[test]
    fn set_dedup_preserves_order() {
        let mut editor = CustomCollectionEditor::new(CollectionType::Set);
        editor.set_as_text("c, a, b, a, c").unwrap();
        assert_eq!(editor.value, vec!["c", "a", "b"]);
    }

    #[test]
    fn get_as_text_empty() {
        let editor = CustomCollectionEditor::new(CollectionType::List);
        assert_eq!(editor.get_as_text(), Some("".to_string()));
    }

    #[test]
    fn len_and_is_empty() {
        let mut editor = CustomCollectionEditor::new(CollectionType::List);
        assert!(editor.is_empty());
        assert_eq!(editor.len(), 0);
        editor.set_as_text("a, b").unwrap();
        assert!(!editor.is_empty());
        assert_eq!(editor.len(), 2);
    }
}
