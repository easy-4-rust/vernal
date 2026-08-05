//! PropertyEditorRegistry — 对应 Spring `org.springframework.beans.PropertyEditorRegistry`。
//!
//! 属性编辑器注册表接口。

use std::any::TypeId;

use crate::property_editor::PropertyEditor;

/// 属性编辑器注册表接口。
///
/// 对应 Java 接口：`org.springframework.beans.PropertyEditorRegistry`。
///
/// 提供了注册和查找属性编辑器的功能。
pub trait PropertyEditorRegistry: Send + Sync {
    /// 注册自定义属性编辑器。
    ///
    /// 对应 Java 方法：`void registerCustomEditor(Class<?> requiredType, PropertyEditor propertyEditor)`
    fn register_custom_editor(&mut self, required_type: TypeId, editor: Box<dyn PropertyEditor>);

    /// 注册自定义属性编辑器（带属性路径）。
    ///
    /// 对应 Java 方法：`void registerCustomEditor(Class<?> requiredType, String propertyPath, PropertyEditor propertyEditor)`
    fn register_custom_editor_for_path(
        &mut self,
        required_type: TypeId,
        property_path: &str,
        editor: Box<dyn PropertyEditor>,
    );

    /// 查找自定义属性编辑器。
    ///
    /// 对应 Java 方法：`PropertyEditor findCustomEditor(Class<?> requiredType, String propertyPath)`
    fn find_custom_editor<'a>(
        &'a self,
        required_type: TypeId,
        property_path: Option<&str>,
    ) -> Option<&'a dyn PropertyEditor>;

    /// 查找自定义属性编辑器（可变引用）。
    fn find_custom_editor_mut<'a>(
        &'a mut self,
        required_type: TypeId,
        property_path: Option<&str>,
    ) -> Option<&'a mut (dyn PropertyEditor + 'a)>;

    /// 检查是否有指定类型的自定义属性编辑器。
    ///
    /// 对应 Java 方法：`boolean hasCustomEditorForElement(String propertyPath)`
    fn has_custom_editor(&self, required_type: TypeId, property_path: Option<&str>) -> bool;
}

/// PropertyEditorRegistry 的默认实现。
pub struct SimplePropertyEditorRegistry {
    /// 按类型注册的编辑器。
    type_editors: std::collections::HashMap<TypeId, Box<dyn PropertyEditor>>,
    /// 按类型+路径注册的编辑器。
    path_editors: std::collections::HashMap<(TypeId, String), Box<dyn PropertyEditor>>,
}

impl SimplePropertyEditorRegistry {
    /// 创建一个新的 SimplePropertyEditorRegistry。
    pub fn new() -> Self {
        Self {
            type_editors: std::collections::HashMap::new(),
            path_editors: std::collections::HashMap::new(),
        }
    }
}

impl Default for SimplePropertyEditorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorRegistry for SimplePropertyEditorRegistry {
    fn register_custom_editor(&mut self, required_type: TypeId, editor: Box<dyn PropertyEditor>) {
        self.type_editors.insert(required_type, editor);
    }

    fn register_custom_editor_for_path(
        &mut self,
        required_type: TypeId,
        property_path: &str,
        editor: Box<dyn PropertyEditor>,
    ) {
        self.path_editors
            .insert((required_type, property_path.to_string()), editor);
    }

    fn find_custom_editor<'a>(
        &'a self,
        required_type: TypeId,
        property_path: Option<&str>,
    ) -> Option<&'a dyn PropertyEditor> {
        // 先查找路径特定的编辑器
        if let Some(path) = property_path {
            if let Some(editor) = self.path_editors.get(&(required_type, path.to_string())) {
                return Some(editor.as_ref());
            }
        }
        // 再查找类型级别的编辑器
        self.type_editors.get(&required_type).map(|e| e.as_ref())
    }

    fn find_custom_editor_mut<'a>(
        &'a mut self,
        required_type: TypeId,
        property_path: Option<&str>,
    ) -> Option<&'a mut (dyn PropertyEditor + 'a)> {
        // 先查找路径特定的编辑器
        if let Some(path) = property_path {
            if let Some(editor) = self
                .path_editors
                .get_mut(&(required_type, path.to_string()))
            {
                return Some(&mut **editor);
            }
        }
        // 再查找类型级别的编辑器
        if let Some(editor) = self.type_editors.get_mut(&required_type) {
            return Some(&mut **editor);
        }
        None
    }

    fn has_custom_editor(&self, required_type: TypeId, property_path: Option<&str>) -> bool {
        self.find_custom_editor(required_type, property_path)
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_editor::PropertyEditor;
    use std::any::Any;
    use std::sync::Arc;

    // 创建一个测试用的 PropertyEditor
    struct TestEditor {
        value: Option<String>,
    }

    impl TestEditor {
        fn new() -> Self {
            Self { value: None }
        }
    }

    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }

        fn set_as_text(
            &mut self,
            text: &str,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.value = Some(text.to_string());
            Ok(())
        }

        fn get_as_text(&self) -> Option<String> {
            self.value.clone()
        }

        fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
            if let Some(s) = value.downcast_ref::<String>() {
                self.value = Some(s.clone());
            }
        }

        fn get_value(&self) -> Option<&dyn Any> {
            self.value.as_ref().map(|v| v as &dyn Any)
        }

        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    #[test]
    fn test_register_and_find() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let editor = Box::new(TestEditor::new());
        registry.register_custom_editor(TypeId::of::<String>(), editor);

        assert!(registry.has_custom_editor(TypeId::of::<String>(), None));
        assert!(!registry.has_custom_editor(TypeId::of::<i32>(), None));
    }

    #[test]
    fn test_register_for_path() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let editor = Box::new(TestEditor::new());
        registry.register_custom_editor_for_path(TypeId::of::<String>(), "name", editor);

        assert!(registry.has_custom_editor(TypeId::of::<String>(), Some("name")));
        assert!(!registry.has_custom_editor(TypeId::of::<String>(), Some("other")));
    }

    #[test]
    fn test_find_editor_returns_none_when_empty() {
        let registry = SimplePropertyEditorRegistry::new();
        assert!(
            registry
                .find_custom_editor(TypeId::of::<String>(), None)
                .is_none()
        );
    }

    #[test]
    fn test_find_editor_by_type() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let mut editor = TestEditor::new();
        editor.set_as_text("test_value").unwrap();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(editor));

        let found = registry.find_custom_editor(TypeId::of::<String>(), None);
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_as_text(), Some("test_value".to_string()));
    }

    #[test]
    fn test_find_editor_by_path_takes_priority() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let mut type_editor = TestEditor::new();
        type_editor.set_as_text("type_level").unwrap();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(type_editor));

        let mut path_editor = TestEditor::new();
        path_editor.set_as_text("path_level").unwrap();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(path_editor),
        );

        // Path-specific editor takes priority
        let found = registry.find_custom_editor(TypeId::of::<String>(), Some("name"));
        assert_eq!(found.unwrap().get_as_text(), Some("path_level".to_string()));

        // Without path, returns type-level editor
        let found = registry.find_custom_editor(TypeId::of::<String>(), None);
        assert_eq!(found.unwrap().get_as_text(), Some("type_level".to_string()));
    }

    #[test]
    fn test_find_editor_mut_by_path() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let mut editor = TestEditor::new();
        editor.set_as_text("initial").unwrap();
        registry.register_custom_editor_for_path(TypeId::of::<String>(), "field", Box::new(editor));

        let found = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("field"));
        assert!(found.is_some());
        found.unwrap().set_as_text("modified").unwrap();

        // Verify modification
        let found = registry.find_custom_editor(TypeId::of::<String>(), Some("field"));
        assert_eq!(found.unwrap().get_as_text(), Some("modified".to_string()));
    }

    #[test]
    fn test_find_editor_mut_by_type() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let editor = TestEditor::new();
        registry.register_custom_editor(TypeId::of::<i32>(), Box::new(editor));

        let found = registry.find_custom_editor_mut(TypeId::of::<i32>(), None);
        assert!(found.is_some());
    }

    #[test]
    fn test_find_editor_mut_returns_none_when_empty() {
        let mut registry = SimplePropertyEditorRegistry::new();
        assert!(
            registry
                .find_custom_editor_mut(TypeId::of::<String>(), None)
                .is_none()
        );
    }

    #[test]
    fn test_has_custom_editor_with_none_path() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let editor = TestEditor::new();
        registry.register_custom_editor(TypeId::of::<bool>(), Box::new(editor));

        assert!(registry.has_custom_editor(TypeId::of::<bool>(), None));
        assert!(!registry.has_custom_editor(TypeId::of::<i32>(), None));
    }

    #[test]
    fn test_register_overwrites_previous() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let mut editor1 = TestEditor::new();
        editor1.set_as_text("first").unwrap();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(editor1));

        let mut editor2 = TestEditor::new();
        editor2.set_as_text("second").unwrap();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(editor2));

        let found = registry.find_custom_editor(TypeId::of::<String>(), None);
        assert_eq!(found.unwrap().get_as_text(), Some("second".to_string()));
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn find_editor_mut_returns_none_for_missing() {
        let mut registry = SimplePropertyEditorRegistry::new();
        assert!(
            registry
                .find_custom_editor_mut(TypeId::of::<i32>(), None)
                .is_none()
        );
    }

    #[test]
    fn find_editor_mut_path_not_found_falls_back_to_type() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let found = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("nonexistent"));
        assert!(found.is_some());
    }

    #[test]
    fn find_editor_path_not_found_returns_none() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "field1",
            Box::new(TestEditor::new()),
        );
        let found = registry.find_custom_editor(TypeId::of::<String>(), Some("field2"));
        assert!(found.is_none());
    }

    #[test]
    fn has_custom_editor_with_path_match() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        assert!(registry.has_custom_editor(TypeId::of::<String>(), Some("name")));
        assert!(!registry.has_custom_editor(TypeId::of::<String>(), Some("other")));
    }

    #[test]
    fn has_custom_editor_type_only() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        assert!(registry.has_custom_editor(TypeId::of::<i32>(), Some("any_path")));
        assert!(registry.has_custom_editor(TypeId::of::<i32>(), None));
    }

    #[test]
    fn register_path_editor_overwrites() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let mut editor1 = TestEditor::new();
        editor1.set_as_text("first").unwrap();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "field",
            Box::new(editor1),
        );

        let mut editor2 = TestEditor::new();
        editor2.set_as_text("second").unwrap();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "field",
            Box::new(editor2),
        );

        let found = registry.find_custom_editor(TypeId::of::<String>(), Some("field"));
        assert_eq!(found.unwrap().get_as_text(), Some("second".to_string()));
    }

    #[test]
    fn default_trait_works() {
        let registry = SimplePropertyEditorRegistry::default();
        assert!(
            registry
                .find_custom_editor(TypeId::of::<String>(), None)
                .is_none()
        );
    }

    #[test]
    fn find_editor_with_none_path() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let found = registry.find_custom_editor(TypeId::of::<String>(), None);
        assert!(found.is_some());
    }

    #[test]
    fn find_editor_mut_with_none_path() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let found = registry.find_custom_editor_mut(TypeId::of::<String>(), None);
        assert!(found.is_some());
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn find_editor_mut_path_priority_over_type() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let mut type_editor = TestEditor::new();
        type_editor.set_as_text("type_level").unwrap();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(type_editor));

        let mut path_editor = TestEditor::new();
        path_editor.set_as_text("path_level").unwrap();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(path_editor),
        );

        // Path-specific editor takes priority
        let found = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("name"));
        assert_eq!(found.unwrap().get_as_text(), Some("path_level".to_string()));
    }

    #[test]
    fn find_editor_mut_fallback_to_type() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        // Path not found, falls back to type editor
        let found = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("nonexistent"));
        assert!(found.is_some());
    }

    #[test]
    fn has_custom_editor_path_and_type() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor_for_path(
            TypeId::of::<i32>(),
            "field",
            Box::new(TestEditor::new()),
        );

        assert!(registry.has_custom_editor(TypeId::of::<String>(), None));
        assert!(registry.has_custom_editor(TypeId::of::<i32>(), Some("field")));
        assert!(!registry.has_custom_editor(TypeId::of::<i32>(), Some("other")));
    }

    #[test]
    fn find_editor_path_not_found_no_type_editor() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "field1",
            Box::new(TestEditor::new()),
        );
        let found = registry.find_custom_editor(TypeId::of::<String>(), Some("field2"));
        assert!(found.is_none());
    }

    #[test]
    fn find_editor_mut_path_not_found_no_type_editor() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "field1",
            Box::new(TestEditor::new()),
        );
        let found = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("field2"));
        assert!(found.is_none());
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[test]
    fn find_editor_by_type_when_path_not_found() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let mut editor = TestEditor::new();
        editor.set_as_text("type_level").unwrap();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(editor));
        // Path doesn't exist, falls back to type editor
        let found = registry.find_custom_editor(TypeId::of::<String>(), Some("nonexistent"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_as_text(), Some("type_level".to_string()));
    }

    #[test]
    fn find_editor_mut_by_type_when_path_not_found() {
        let mut registry = SimplePropertyEditorRegistry::new();
        let mut editor = TestEditor::new();
        editor.set_as_text("type_level").unwrap();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(editor));
        // Path doesn't exist, falls back to type editor
        let found = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("nonexistent"));
        assert!(found.is_some());
    }

    #[test]
    fn has_custom_editor_with_path_and_type() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor_for_path(
            TypeId::of::<i32>(),
            "field",
            Box::new(TestEditor::new()),
        );
        assert!(registry.has_custom_editor(TypeId::of::<String>(), None));
        assert!(registry.has_custom_editor(TypeId::of::<String>(), Some("any")));
        assert!(registry.has_custom_editor(TypeId::of::<i32>(), Some("field")));
        assert!(!registry.has_custom_editor(TypeId::of::<i32>(), Some("other")));
    }

    #[test]
    fn register_multiple_editors_different_types() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        registry.register_custom_editor(TypeId::of::<bool>(), Box::new(TestEditor::new()));
        assert!(registry.has_custom_editor(TypeId::of::<String>(), None));
        assert!(registry.has_custom_editor(TypeId::of::<i32>(), None));
        assert!(registry.has_custom_editor(TypeId::of::<bool>(), None));
    }

    #[test]
    fn register_multiple_path_editors() {
        let mut registry = SimplePropertyEditorRegistry::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "field1",
            Box::new(TestEditor::new()),
        );
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "field2",
            Box::new(TestEditor::new()),
        );
        assert!(registry.has_custom_editor(TypeId::of::<String>(), Some("field1")));
        assert!(registry.has_custom_editor(TypeId::of::<String>(), Some("field2")));
        assert!(!registry.has_custom_editor(TypeId::of::<String>(), Some("field3")));
    }
}
