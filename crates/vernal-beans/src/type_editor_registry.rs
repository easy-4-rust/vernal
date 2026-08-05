//! TypeEditorRegistry — Spring 风格类型编辑器注册表。
//!
//! 对应 Java 类：`org.springframework.beans.PropertyEditorRegistry` 的扩展。
//!
//! 在 Spring 中，`PropertyEditorRegistry` 管理类型到属性编辑器的映射。
//! `TypeEditorRegistry` 在此基础上提供按类型名称查找编辑器的能力，
//! 支持别名和类型继承链查找。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::property_editor::PropertyEditor;

/// 类型编辑器注册表。
///
/// 对应 Spring `PropertyEditorRegistry` 的扩展功能。
///
/// 支持按 `TypeId` 和类型名称注册/查找属性编辑器。
/// 提供别名机制，允许为同一类型注册多个名称。
pub struct TypeEditorRegistry {
    /// 按 TypeId 注册的编辑器
    editors_by_type: RwLock<HashMap<TypeId, Arc<dyn PropertyEditor>>>,
    /// 按类型名称注册的编辑器
    editors_by_name: RwLock<HashMap<String, Arc<dyn PropertyEditor>>>,
    /// 类型别名映射（别名 -> 标准名称）
    type_aliases: RwLock<HashMap<String, String>>,
}

impl TypeEditorRegistry {
    /// 创建空的类型编辑器注册表。
    pub fn new() -> Self {
        Self {
            editors_by_type: RwLock::new(HashMap::new()),
            editors_by_name: RwLock::new(HashMap::new()),
            type_aliases: RwLock::new(HashMap::new()),
        }
    }

    /// 按 TypeId 注册编辑器。
    pub fn register_editor(&self, type_id: TypeId, editor: Arc<dyn PropertyEditor>) {
        self.editors_by_type
            .write()
            .unwrap()
            .insert(type_id, editor);
    }

    /// 按类型名称注册编辑器。
    pub fn register_editor_by_name(&self, type_name: &str, editor: Arc<dyn PropertyEditor>) {
        self.editors_by_name
            .write()
            .unwrap()
            .insert(type_name.to_string(), editor);
    }

    /// 注册类型别名。
    ///
    /// 例如将 "int" 和 "java.lang.Integer" 都映射到 "i32"。
    pub fn register_alias(&self, alias: &str, canonical_name: &str) {
        self.type_aliases
            .write()
            .unwrap()
            .insert(alias.to_string(), canonical_name.to_string());
    }

    /// 按 TypeId 查找编辑器。
    pub fn find_editor_by_type(&self, type_id: TypeId) -> Option<Arc<dyn PropertyEditor>> {
        self.editors_by_type.read().unwrap().get(&type_id).cloned()
    }

    /// 按类型名称查找编辑器（支持别名解析）。
    pub fn find_editor_by_name(&self, type_name: &str) -> Option<Arc<dyn PropertyEditor>> {
        // 先直接查找
        if let Some(editor) = self.editors_by_name.read().unwrap().get(type_name) {
            return Some(Arc::clone(editor));
        }

        // 尝试别名解析
        let aliases = self.type_aliases.read().unwrap();
        if let Some(canonical) = aliases.get(type_name) {
            return self.editors_by_name.read().unwrap().get(canonical).cloned();
        }

        None
    }

    /// 解析类型别名，返回标准名称。
    pub fn resolve_alias(&self, alias: &str) -> Option<String> {
        self.type_aliases.read().unwrap().get(alias).cloned()
    }

    /// 按 TypeId 的编辑器数量。
    pub fn editor_count_by_type(&self) -> usize {
        self.editors_by_type.read().unwrap().len()
    }

    /// 按名称的编辑器数量。
    pub fn editor_count_by_name(&self) -> usize {
        self.editors_by_name.read().unwrap().len()
    }

    /// 类型别名数量。
    pub fn alias_count(&self) -> usize {
        self.type_aliases.read().unwrap().len()
    }

    /// 是否包含指定 TypeId 的编辑器。
    pub fn has_editor_for_type(&self, type_id: TypeId) -> bool {
        self.editors_by_type.read().unwrap().contains_key(&type_id)
    }

    /// 是否包含指定名称的编辑器（含别名）。
    pub fn has_editor_for_name(&self, type_name: &str) -> bool {
        if self.editors_by_name.read().unwrap().contains_key(type_name) {
            return true;
        }
        if let Some(canonical) = self.type_aliases.read().unwrap().get(type_name) {
            return self.editors_by_name.read().unwrap().contains_key(canonical);
        }
        false
    }

    /// 清空注册表。
    pub fn clear(&self) {
        self.editors_by_type.write().unwrap().clear();
        self.editors_by_name.write().unwrap().clear();
        self.type_aliases.write().unwrap().clear();
    }
}

impl Default for TypeEditorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TypeEditorRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeEditorRegistry")
            .field("editors_by_type", &self.editor_count_by_type())
            .field("editors_by_name", &self.editor_count_by_name())
            .field("aliases", &self.alias_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::sync::Arc;

    #[derive(Debug)]
    struct StubEditor {
        name: String,
    }

    impl StubEditor {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl PropertyEditor for StubEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }

        fn set_as_text(
            &mut self,
            _text: &str,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> {
            Some(self.name.clone())
        }
        fn set_value(&mut self, _value: Arc<dyn Any + Send + Sync>) {}
        fn get_value(&self) -> Option<&dyn std::any::Any> {
            Some(&self.name)
        }
        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    #[test]
    fn register_and_find_by_type() {
        let registry = TypeEditorRegistry::new();
        let editor = Arc::new(StubEditor::new("int-editor"));
        registry.register_editor(TypeId::of::<i32>(), editor);

        assert!(registry.has_editor_for_type(TypeId::of::<i32>()));
        assert_eq!(registry.editor_count_by_type(), 1);

        let found = registry.find_editor_by_type(TypeId::of::<i32>()).unwrap();
        assert_eq!(found.get_as_text(), Some("int-editor".to_string()));
    }

    #[test]
    fn register_and_find_by_name() {
        let registry = TypeEditorRegistry::new();
        let editor = Arc::new(StubEditor::new("string-editor"));
        registry.register_editor_by_name("String", editor);

        assert!(registry.has_editor_for_name("String"));
        let found = registry.find_editor_by_name("String").unwrap();
        assert_eq!(found.get_as_text(), Some("string-editor".to_string()));
    }

    #[test]
    fn alias_resolution() {
        let registry = TypeEditorRegistry::new();
        let editor = Arc::new(StubEditor::new("int-editor"));
        registry.register_editor_by_name("i32", editor);
        registry.register_alias("int", "i32");
        registry.register_alias("java.lang.Integer", "i32");

        assert!(registry.has_editor_for_name("int"));
        assert!(registry.has_editor_for_name("java.lang.Integer"));

        let found = registry.find_editor_by_name("int").unwrap();
        assert_eq!(found.get_as_text(), Some("int-editor".to_string()));
    }

    #[test]
    fn clear_removes_all() {
        let registry = TypeEditorRegistry::new();
        registry.register_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("a")));
        registry.register_editor_by_name("String", Arc::new(StubEditor::new("b")));
        registry.register_alias("int", "i32");

        registry.clear();
        assert_eq!(registry.editor_count_by_type(), 0);
        assert_eq!(registry.editor_count_by_name(), 0);
        assert_eq!(registry.alias_count(), 0);
    }

    #[test]
    fn default_trait_creates_empty() {
        let registry = TypeEditorRegistry::default();
        assert_eq!(registry.editor_count_by_type(), 0);
        assert_eq!(registry.editor_count_by_name(), 0);
        assert_eq!(registry.alias_count(), 0);
    }

    #[test]
    fn find_editor_by_type_not_found() {
        let registry = TypeEditorRegistry::new();
        assert!(registry.find_editor_by_type(TypeId::of::<i32>()).is_none());
    }

    #[test]
    fn find_editor_by_name_not_found() {
        let registry = TypeEditorRegistry::new();
        assert!(registry.find_editor_by_name("nonexistent").is_none());
    }

    #[test]
    fn find_editor_by_name_alias_not_found() {
        let registry = TypeEditorRegistry::new();
        registry.register_alias("int", "i32");
        // "int" -> "i32" but "i32" editor not registered
        assert!(registry.find_editor_by_name("int").is_none());
    }

    #[test]
    fn find_editor_by_name_alias_resolves() {
        let registry = TypeEditorRegistry::new();
        registry.register_editor_by_name("i32", Arc::new(StubEditor::new("int-editor")));
        registry.register_alias("int", "i32");
        registry.register_alias("java.lang.Integer", "i32");

        let found = registry.find_editor_by_name("java.lang.Integer").unwrap();
        assert_eq!(found.get_as_text(), Some("int-editor".to_string()));
    }

    #[test]
    fn resolve_alias_found() {
        let registry = TypeEditorRegistry::new();
        registry.register_alias("int", "i32");
        assert_eq!(registry.resolve_alias("int"), Some("i32".to_string()));
    }

    #[test]
    fn resolve_alias_not_found() {
        let registry = TypeEditorRegistry::new();
        assert!(registry.resolve_alias("nonexistent").is_none());
    }

    #[test]
    fn has_editor_for_type_false() {
        let registry = TypeEditorRegistry::new();
        assert!(!registry.has_editor_for_type(TypeId::of::<i32>()));
    }

    #[test]
    fn has_editor_for_name_direct() {
        let registry = TypeEditorRegistry::new();
        registry.register_editor_by_name("MyType", Arc::new(StubEditor::new("e")));
        assert!(registry.has_editor_for_name("MyType"));
    }

    #[test]
    fn has_editor_for_name_via_alias() {
        let registry = TypeEditorRegistry::new();
        registry.register_editor_by_name("i32", Arc::new(StubEditor::new("e")));
        registry.register_alias("int", "i32");
        assert!(registry.has_editor_for_name("int"));
    }

    #[test]
    fn has_editor_for_name_not_found() {
        let registry = TypeEditorRegistry::new();
        assert!(!registry.has_editor_for_name("nonexistent"));
    }

    #[test]
    fn has_editor_for_name_alias_without_editor() {
        let registry = TypeEditorRegistry::new();
        registry.register_alias("int", "i32");
        // "int" -> "i32" but no editor for "i32"
        assert!(!registry.has_editor_for_name("int"));
    }

    #[test]
    fn register_editor_overwrites() {
        let registry = TypeEditorRegistry::new();
        registry.register_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("first")));
        registry.register_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("second")));
        assert_eq!(registry.editor_count_by_type(), 1);
        let found = registry.find_editor_by_type(TypeId::of::<i32>()).unwrap();
        assert_eq!(found.get_as_text(), Some("second".to_string()));
    }

    #[test]
    fn register_editor_by_name_overwrites() {
        let registry = TypeEditorRegistry::new();
        registry.register_editor_by_name("Type", Arc::new(StubEditor::new("first")));
        registry.register_editor_by_name("Type", Arc::new(StubEditor::new("second")));
        assert_eq!(registry.editor_count_by_name(), 1);
    }

    #[test]
    fn debug_format() {
        let registry = TypeEditorRegistry::new();
        let debug = format!("{:?}", registry);
        assert!(debug.contains("TypeEditorRegistry"));
    }

    #[test]
    fn register_alias_overwrites() {
        let registry = TypeEditorRegistry::new();
        registry.register_alias("int", "i32");
        registry.register_alias("int", "integer");
        assert_eq!(registry.alias_count(), 1);
        assert_eq!(registry.resolve_alias("int"), Some("integer".to_string()));
    }
}
