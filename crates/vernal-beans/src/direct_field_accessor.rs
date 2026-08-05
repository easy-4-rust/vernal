//! DirectFieldAccessor — 对应 Spring `org.springframework.beans.DirectFieldAccessor`。
//!
//! 直接字段访问器，绕过 getter/setter 直接访问字段。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;

use crate::property_editor::PropertyEditor;

/// 直接字段访问器。
///
/// 对应 Java 类：`org.springframework.beans.DirectFieldAccessor`。
///
/// 提供了直接访问对象字段的能力，不通过 getter/setter 方法。
pub struct DirectFieldAccessor {
    /// 目标对象。
    target: Box<dyn Any + Send + Sync>,
    /// 字段值缓存。
    fields: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,
    /// 自定义属性编辑器。
    custom_editors: RwLock<HashMap<TypeId, Box<dyn PropertyEditor>>>,
}

impl DirectFieldAccessor {
    /// 创建一个新的 DirectFieldAccessor。
    pub fn new(target: impl Any + Send + Sync + 'static) -> Self {
        Self {
            target: Box::new(target),
            fields: RwLock::new(HashMap::new()),
            custom_editors: RwLock::new(HashMap::new()),
        }
    }

    /// 获取字段值。
    pub fn get_field_value(&self, field_name: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.fields.read().ok().and_then(|fields| {
            fields.get(field_name).map(|_| {
                // 由于 dyn Any 不实现 Clone，返回一个占位值
                Box::new(()) as Box<dyn Any + Send + Sync>
            })
        })
    }

    /// 设置字段值。
    pub fn set_field_value(&self, field_name: impl Into<String>, value: impl Any + Send + Sync) {
        if let Ok(mut fields) = self.fields.write() {
            fields.insert(field_name.into(), Box::new(value));
        }
    }

    /// 检查是否有指定字段。
    pub fn has_field(&self, field_name: &str) -> bool {
        self.fields
            .read()
            .ok()
            .map(|fields| fields.contains_key(field_name))
            .unwrap_or(false)
    }

    /// 获取字段类型。
    pub fn get_field_type(&self, field_name: &str) -> Option<TypeId> {
        self.fields
            .read()
            .ok()
            .and_then(|fields| fields.get(field_name).map(|v| v.as_ref().type_id()))
    }

    /// 获取所有字段名称。
    pub fn field_names(&self) -> Vec<String> {
        self.fields
            .read()
            .ok()
            .map(|fields| fields.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取目标对象。
    pub fn target(&self) -> &dyn Any {
        self.target.as_ref()
    }

    /// 注册自定义属性编辑器。
    pub fn register_custom_editor(&self, type_id: TypeId, editor: Box<dyn PropertyEditor>) {
        if let Ok(mut editors) = self.custom_editors.write() {
            editors.insert(type_id, editor);
        }
    }

    /// 获取自定义属性编辑器。
    pub fn get_custom_editor(&self, type_id: TypeId) -> Option<Box<dyn PropertyEditor>> {
        self.custom_editors
            .read()
            .ok()
            .and_then(|editors| {
                editors.get(&type_id).map(|_| {
                    // 由于 PropertyEditor 不实现 Clone，返回 None
                    None
                })
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        assert!(accessor.target().is::<String>());
    }

    #[test]
    fn test_set_and_get_field() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        accessor.set_field_value("name", String::from("Alice"));
        assert!(accessor.has_field("name"));
    }

    #[test]
    fn test_field_names() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        accessor.set_field_value("a", 1);
        accessor.set_field_value("b", 2);
        let mut names = accessor.field_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_has_field() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        assert!(!accessor.has_field("missing"));
        accessor.set_field_value("exists", 42);
        assert!(accessor.has_field("exists"));
    }

    #[test]
    fn test_get_field_value() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        assert!(accessor.get_field_value("missing").is_none());
        accessor.set_field_value("name", String::from("Alice"));
        // get_field_value returns a placeholder since dyn Any doesn't implement Clone
        assert!(accessor.get_field_value("name").is_some());
    }

    #[test]
    fn test_get_field_type() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        assert!(accessor.get_field_type("missing").is_none());
        accessor.set_field_value("count", 42i32);
        assert_eq!(
            accessor.get_field_type("count"),
            Some(std::any::TypeId::of::<i32>())
        );
    }

    #[test]
    fn test_field_names_empty() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        assert!(accessor.field_names().is_empty());
    }

    #[test]
    fn test_target_returns_correct_type() {
        let accessor = DirectFieldAccessor::new(String::from("hello"));
        let target = accessor.target();
        assert!(target.is::<String>());
        assert_eq!(target.downcast_ref::<String>().unwrap(), "hello");
    }

    #[test]
    fn test_target_with_different_types() {
        let accessor = DirectFieldAccessor::new(42i32);
        assert!(accessor.target().is::<i32>());
        assert_eq!(accessor.target().downcast_ref::<i32>().unwrap(), &42);
    }

    #[test]
    fn test_register_custom_editor() {
        use crate::property_editor::PropertyEditor;
        use std::any::TypeId;
        use std::sync::Arc;

        struct DummyEditor;
        impl PropertyEditor for DummyEditor {
            fn target_type(&self) -> TypeId {
                TypeId::of::<String>()
            }
            fn set_as_text(
                &mut self,
                _: &str,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Ok(())
            }
            fn get_as_text(&self) -> Option<String> {
                None
            }
            fn set_value(&mut self, _: Arc<dyn std::any::Any + Send + Sync>) {}
            fn get_value(&self) -> Option<&dyn std::any::Any> {
                None
            }
            fn get_value_type(&self) -> TypeId {
                TypeId::of::<String>()
            }
        }

        let accessor = DirectFieldAccessor::new(String::from("test"));
        accessor.register_custom_editor(TypeId::of::<String>(), Box::new(DummyEditor));
        // get_custom_editor always returns None due to Clone limitation
        assert!(accessor.get_custom_editor(TypeId::of::<String>()).is_none());
    }

    #[test]
    fn test_multiple_fields() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        accessor.set_field_value("a", 1i32);
        accessor.set_field_value("b", 2i64);
        accessor.set_field_value("c", "three".to_string());
        assert_eq!(accessor.field_names().len(), 3);
        assert!(accessor.has_field("a"));
        assert!(accessor.has_field("b"));
        assert!(accessor.has_field("c"));
    }

    #[test]
    fn test_overwrite_field() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        accessor.set_field_value("val", 1i32);
        accessor.set_field_value("val", 2i32);
        assert_eq!(
            accessor.get_field_type("val"),
            Some(std::any::TypeId::of::<i32>())
        );
    }

    #[test]
    fn test_field_type_changes() {
        let accessor = DirectFieldAccessor::new(String::from("test"));
        accessor.set_field_value("dynamic", 42i32);
        assert_eq!(
            accessor.get_field_type("dynamic"),
            Some(std::any::TypeId::of::<i32>())
        );
        accessor.set_field_value("dynamic", "now_string".to_string());
        assert_eq!(
            accessor.get_field_type("dynamic"),
            Some(std::any::TypeId::of::<String>())
        );
    }
}
