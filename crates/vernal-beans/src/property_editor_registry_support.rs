//! PropertyEditorRegistrySupport — 对应 Spring `org.springframework.beans.PropertyEditorRegistrySupport`。
//!
/// PropertyEditorRegistry 的基类实现。

use std::any::TypeId;
use std::collections::HashMap;

use crate::property_editor::PropertyEditor;
use crate::property_editor_registry::PropertyEditorRegistry;

/// PropertyEditorRegistry 的基类实现。
///
/// 对应 Java 类：`org.springframework.beans.PropertyEditorRegistrySupport`。
pub struct PropertyEditorRegistrySupport {
    custom_editors: HashMap<TypeId, Box<dyn PropertyEditor>>,
    custom_editors_for_path: HashMap<(TypeId, String), Box<dyn PropertyEditor>>,
    default_editors: HashMap<TypeId, Box<dyn PropertyEditor>>,
    default_editors_active: bool,
}

impl PropertyEditorRegistrySupport {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self {
            custom_editors: HashMap::new(),
            custom_editors_for_path: HashMap::new(),
            default_editors: HashMap::new(),
            default_editors_active: true,
        }
    }

    /// 注册默认编辑器。
    pub fn register_default_editor(&mut self, required_type: TypeId, editor: Box<dyn PropertyEditor>) {
        self.default_editors.insert(required_type, editor);
    }

    /// 获取默认编辑器。
    pub fn get_default_editor(&self, required_type: TypeId) -> Option<&dyn PropertyEditor> {
        if self.default_editors_active {
            self.default_editors.get(&required_type).map(|e| e.as_ref())
        } else {
            None
        }
    }

    /// 判断是否默认编辑器。
    pub fn has_default_editor(&self, required_type: TypeId) -> bool {
        self.default_editors_active && self.default_editors.contains_key(&required_type)
    }

    /// 覆盖默认editors。
    pub fn override_default_editors(&mut self) {
        self.default_editors_active = false;
    }

    /// 恢复默认editors。
    pub fn restore_default_editors(&mut self) {
        self.default_editors_active = true;
    }

    /// 获取自定义编辑器数量。
    pub fn custom_editor_count(&self) -> usize {
        self.custom_editors.len() + self.custom_editors_for_path.len()
    }
}

impl Default for PropertyEditorRegistrySupport {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditorRegistry for PropertyEditorRegistrySupport {
    fn register_custom_editor(&mut self, required_type: TypeId, editor: Box<dyn PropertyEditor>) {
        self.custom_editors.insert(required_type, editor);
    }

    fn register_custom_editor_for_path(&mut self, required_type: TypeId, property_path: &str, editor: Box<dyn PropertyEditor>) {
        self.custom_editors_for_path.insert((required_type, property_path.to_string()), editor);
    }

    fn find_custom_editor<'a>(&'a self, required_type: TypeId, property_path: Option<&str>) -> Option<&'a dyn PropertyEditor> {
        if let Some(path) = property_path {
            if let Some(editor) = self.custom_editors_for_path.get(&(required_type, path.to_string())) {
                return Some(editor.as_ref());
            }
        }
        if let Some(editor) = self.custom_editors.get(&required_type) {
            return Some(editor.as_ref());
        }
        if self.default_editors_active {
            self.default_editors.get(&required_type).map(|e| e.as_ref())
        } else {
            None
        }
    }

    fn find_custom_editor_mut<'a>(&'a mut self, required_type: TypeId, property_path: Option<&str>) -> Option<&'a mut (dyn PropertyEditor + 'a)> {
        if let Some(path) = property_path {
            if let Some(editor) = self.custom_editors_for_path.get_mut(&(required_type, path.to_string())) {
                return Some(&mut **editor);
            }
        }
        if let Some(editor) = self.custom_editors.get_mut(&required_type) {
            return Some(&mut **editor);
        }
        if self.default_editors_active {
            if let Some(editor) = self.default_editors.get_mut(&required_type) {
                return Some(&mut **editor);
            }
        }
        None
    }

    fn has_custom_editor(&self, required_type: TypeId, property_path: Option<&str>) -> bool {
        self.find_custom_editor(required_type, property_path).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_editor::PropertyEditor;
    use std::any::Any;
    use std::sync::Arc;

    struct TestEditor {
        value: Option<String>,
    }

    impl TestEditor {
        fn new() -> Self {
            Self { value: None }
        }
    }

    impl PropertyEditor for TestEditor {
        fn target_type(&self) -> TypeId { TypeId::of::<String>() }
        fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.value = Some(text.to_string());
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> { self.value.clone() }
        fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
            if let Some(s) = value.downcast_ref::<String>() {
                self.value = Some(s.clone());
            }
        }
        fn get_value(&self) -> Option<&dyn Any> { self.value.as_ref().map(|v| v as &dyn Any) }
        fn get_value_type(&self) -> TypeId { TypeId::of::<String>() }
    }

    #[test]
    fn test_new() {
        let registry = PropertyEditorRegistrySupport::new();
        assert!(registry.default_editors_active);
    }

    #[test]
    fn test_register_custom_editor() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        assert!(registry.has_custom_editor(TypeId::of::<String>(), None));
        assert_eq!(registry.custom_editor_count(), 1);
    }

    #[test]
    fn test_override_default_editors() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.override_default_editors();
        registry.restore_default_editors();
    }

    // ── Default editor tests ─────────────────────────────────────────────

    #[test]
    fn register_and_get_default_editor() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        let editor = registry.get_default_editor(TypeId::of::<i32>());
        assert!(editor.is_some());
    }

    #[test]
    fn get_default_editor_not_registered() {
        let registry = PropertyEditorRegistrySupport::new();
        let editor = registry.get_default_editor(TypeId::of::<i32>());
        assert!(editor.is_none());
    }

    #[test]
    fn get_default_editor_when_overridden() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        registry.override_default_editors();
        let editor = registry.get_default_editor(TypeId::of::<i32>());
        assert!(editor.is_none());
    }

    #[test]
    fn get_default_editor_after_restore() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        registry.override_default_editors();
        registry.restore_default_editors();
        let editor = registry.get_default_editor(TypeId::of::<i32>());
        assert!(editor.is_some());
    }

    #[test]
    fn has_default_editor_registered() {
        let mut registry = PropertyEditorRegistrySupport::new();
        assert!(!registry.has_default_editor(TypeId::of::<i32>()));
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        assert!(registry.has_default_editor(TypeId::of::<i32>()));
    }

    #[test]
    fn has_default_editor_when_overridden() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        registry.override_default_editors();
        assert!(!registry.has_default_editor(TypeId::of::<i32>()));
    }

    // ── Custom editor tests ──────────────────────────────────────────────

    #[test]
    fn register_custom_editor_for_path() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        assert_eq!(registry.custom_editor_count(), 1);
    }

    #[test]
    fn custom_editor_count_multiple() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor_for_path(
            TypeId::of::<i32>(),
            "age",
            Box::new(TestEditor::new()),
        );
        assert_eq!(registry.custom_editor_count(), 2);
    }

    #[test]
    fn find_custom_editor_by_type() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor(TypeId::of::<String>(), None);
        assert!(editor.is_some());
    }

    #[test]
    fn find_custom_editor_not_found() {
        let registry = PropertyEditorRegistrySupport::new();
        let editor = registry.find_custom_editor(TypeId::of::<String>(), None);
        assert!(editor.is_none());
    }

    #[test]
    fn find_custom_editor_by_path() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        let editor = registry.find_custom_editor(TypeId::of::<String>(), Some("name"));
        assert!(editor.is_some());
    }

    #[test]
    fn find_custom_editor_path_not_found() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        let editor = registry.find_custom_editor(TypeId::of::<String>(), Some("other"));
        assert!(editor.is_none());
    }

    #[test]
    fn find_custom_editor_path_falls_back_to_type() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor(TypeId::of::<String>(), Some("name"));
        assert!(editor.is_some());
    }

    #[test]
    fn find_custom_editor_falls_back_to_default() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor(TypeId::of::<i32>(), None);
        assert!(editor.is_some());
    }

    #[test]
    fn find_custom_editor_no_fallback_when_defaults_overridden() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        registry.override_default_editors();
        let editor = registry.find_custom_editor(TypeId::of::<i32>(), None);
        assert!(editor.is_none());
    }

    // ── has_custom_editor ────────────────────────────────────────────────

    #[test]
    fn has_custom_editor_true() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        assert!(registry.has_custom_editor(TypeId::of::<String>(), None));
    }

    #[test]
    fn has_custom_editor_false() {
        let registry = PropertyEditorRegistrySupport::new();
        assert!(!registry.has_custom_editor(TypeId::of::<String>(), None));
    }

    #[test]
    fn has_custom_editor_with_path() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        assert!(registry.has_custom_editor(TypeId::of::<String>(), Some("name")));
        assert!(!registry.has_custom_editor(TypeId::of::<String>(), Some("other")));
    }

    // ── find_custom_editor_mut ───────────────────────────────────────────

    #[test]
    fn find_custom_editor_mut_by_path() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("name"));
        assert!(editor.is_some());
    }

    #[test]
    fn find_custom_editor_mut_by_type() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), None);
        assert!(editor.is_some());
    }

    #[test]
    fn find_custom_editor_mut_not_found() {
        let mut registry = PropertyEditorRegistrySupport::new();
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), None);
        assert!(editor.is_none());
    }

    #[test]
    fn find_custom_editor_mut_default_editor() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor_mut(TypeId::of::<i32>(), None);
        assert!(editor.is_some());
    }

    #[test]
    fn find_custom_editor_mut_no_default_when_overridden() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        registry.override_default_editors();
        let editor = registry.find_custom_editor_mut(TypeId::of::<i32>(), None);
        assert!(editor.is_none());
    }

    // ── Default trait ────────────────────────────────────────────────────

    #[test]
    fn default_trait_works() {
        let registry = PropertyEditorRegistrySupport::default();
        assert!(registry.default_editors_active);
    }

    // ── Priority: path > type > default ──────────────────────────────────

    #[test]
    fn path_editor_takes_priority_over_type_editor() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "special",
            Box::new(TestEditor::new()),
        );
        // Both exist, path editor should be found first
        let editor = registry.find_custom_editor(TypeId::of::<String>(), Some("special"));
        assert!(editor.is_some());
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn find_editor_mut_path_priority() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "special",
            Box::new(TestEditor::new()),
        );
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("special"));
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_mut_falls_back_to_type() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("nonexistent"));
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_mut_falls_back_to_default() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor_mut(TypeId::of::<i32>(), None);
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_mut_no_default_when_overridden() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        registry.override_default_editors();
        let editor = registry.find_custom_editor_mut(TypeId::of::<i32>(), None);
        assert!(editor.is_none());
    }

    #[test]
    fn find_editor_mut_path_not_found_returns_none() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "field1",
            Box::new(TestEditor::new()),
        );
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("field2"));
        assert!(editor.is_none());
    }

    #[test]
    fn custom_editor_count_with_path_and_type() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor_for_path(
            TypeId::of::<i32>(),
            "age",
            Box::new(TestEditor::new()),
        );
        assert_eq!(registry.custom_editor_count(), 2);
    }

    #[test]
    fn has_default_editor_not_registered() {
        let registry = PropertyEditorRegistrySupport::new();
        assert!(!registry.has_default_editor(TypeId::of::<String>()));
    }

    #[test]
    fn find_custom_editor_with_none_path_and_no_type_editor() {
        let registry = PropertyEditorRegistrySupport::new();
        let editor = registry.find_custom_editor(TypeId::of::<String>(), None);
        assert!(editor.is_none());
    }

    #[test]
    fn find_custom_editor_path_no_match_no_type_editor() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        // Search with a different path and no type editor
        let editor = registry.find_custom_editor(TypeId::of::<String>(), Some("other"));
        assert!(editor.is_none());
    }

    #[test]
    fn find_custom_editor_path_no_match_falls_to_default() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        registry.register_default_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        // Search with a different path: should fall back to default
        let editor = registry.find_custom_editor(TypeId::of::<String>(), Some("other"));
        assert!(editor.is_some());
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn find_editor_mut_path_no_match_no_type_editor() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("other"));
        assert!(editor.is_none());
    }

    #[test]
    fn find_editor_mut_path_no_match_falls_to_default() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        registry.register_default_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("other"));
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_mut_path_no_match_no_default_when_overridden() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        registry.register_default_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.override_default_editors();
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("other"));
        assert!(editor.is_none());
    }

    #[test]
    fn custom_editor_count_empty() {
        let registry = PropertyEditorRegistrySupport::new();
        assert_eq!(registry.custom_editor_count(), 0);
    }

    #[test]
    fn find_editor_mut_path_found() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor_for_path(
            TypeId::of::<String>(),
            "name",
            Box::new(TestEditor::new()),
        );
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("name"));
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_mut_type_found() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), None);
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_mut_default_found() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor_mut(TypeId::of::<i32>(), None);
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_mut_default_not_found_when_overridden() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        registry.override_default_editors();
        let editor = registry.find_custom_editor_mut(TypeId::of::<i32>(), None);
        assert!(editor.is_none());
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[test]
    fn find_editor_path_priority_over_type_and_default() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor_for_path(TypeId::of::<String>(), "special", Box::new(TestEditor::new()));
        // Path editor should be found first
        let editor = registry.find_custom_editor(TypeId::of::<String>(), Some("special"));
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_mut_path_priority_over_type_and_default() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor_for_path(TypeId::of::<String>(), "special", Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), Some("special"));
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_type_priority_over_default() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        // Type editor should be found before default
        let editor = registry.find_custom_editor(TypeId::of::<String>(), None);
        assert!(editor.is_some());
    }

    #[test]
    fn find_editor_mut_type_priority_over_default() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        registry.register_custom_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), None);
        assert!(editor.is_some());
    }

    #[test]
    fn custom_editor_count_with_only_defaults() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        registry.register_default_editor(TypeId::of::<String>(), Box::new(TestEditor::new()));
        // Default editors don't count as custom
        assert_eq!(registry.custom_editor_count(), 0);
    }

    #[test]
    fn has_default_editor_after_override_and_restore() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        assert!(registry.has_default_editor(TypeId::of::<i32>()));
        registry.override_default_editors();
        assert!(!registry.has_default_editor(TypeId::of::<i32>()));
        registry.restore_default_editors();
        assert!(registry.has_default_editor(TypeId::of::<i32>()));
    }

    #[test]
    fn get_default_editor_after_override_and_restore() {
        let mut registry = PropertyEditorRegistrySupport::new();
        registry.register_default_editor(TypeId::of::<i32>(), Box::new(TestEditor::new()));
        assert!(registry.get_default_editor(TypeId::of::<i32>()).is_some());
        registry.override_default_editors();
        assert!(registry.get_default_editor(TypeId::of::<i32>()).is_none());
        registry.restore_default_editors();
        assert!(registry.get_default_editor(TypeId::of::<i32>()).is_some());
    }

    #[test]
    fn find_editor_no_custom_no_default_returns_none() {
        let registry = PropertyEditorRegistrySupport::new();
        let editor = registry.find_custom_editor(TypeId::of::<String>(), None);
        assert!(editor.is_none());
    }

    #[test]
    fn find_editor_mut_no_custom_no_default_returns_none() {
        let mut registry = PropertyEditorRegistrySupport::new();
        let editor = registry.find_custom_editor_mut(TypeId::of::<String>(), None);
        assert!(editor.is_none());
    }
}
