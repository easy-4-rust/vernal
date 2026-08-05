//! PropertyEditorCache — Spring 风格属性编辑器缓存。
//!
//! 对应 Java 类：`org.springframework.beans.PropertyEditorRegistrySupport` 中的缓存机制。
//!
//! 在 Spring 中，属性编辑器按类型缓存以提高查找性能。
//! `PropertyEditorCache` 提供线程安全的类型到编辑器映射缓存。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::property_editor::PropertyEditor;

/// 属性编辑器缓存。
///
/// 对应 Spring `PropertyEditorRegistrySupport` 中的默认编辑器缓存。
///
/// 提供按 `TypeId` 缓存 `PropertyEditor` 实例的能力，
/// 避免每次属性访问时重复创建编辑器。
pub struct PropertyEditorCache {
    /// 自定义编辑器缓存（用户注册的）
    custom_editors: RwLock<HashMap<TypeId, Arc<dyn PropertyEditor>>>,
    /// 默认编辑器缓存（框架内置的）
    default_editors: RwLock<HashMap<TypeId, Arc<dyn PropertyEditor>>>,
    /// 是否已注册默认编辑器
    default_editors_registered: RwLock<bool>,
}

impl PropertyEditorCache {
    /// 创建空的属性编辑器缓存。
    pub fn new() -> Self {
        Self {
            custom_editors: RwLock::new(HashMap::new()),
            default_editors: RwLock::new(HashMap::new()),
            default_editors_registered: RwLock::new(false),
        }
    }

    /// 注册自定义属性编辑器。
    ///
    /// 对应 Spring 的 `PropertyEditorRegistry.registerCustomEditor(Class<?> requiredType, PropertyEditor propertyEditor)`。
    pub fn register_custom_editor(&self, type_id: TypeId, editor: Arc<dyn PropertyEditor>) {
        self.custom_editors.write().unwrap().insert(type_id, editor);
    }

    /// 查找自定义属性编辑器。
    pub fn find_custom_editor(&self, type_id: TypeId) -> Option<Arc<dyn PropertyEditor>> {
        self.custom_editors.read().unwrap().get(&type_id).cloned()
    }

    /// 注册默认属性编辑器。
    pub fn register_default_editor(&self, type_id: TypeId, editor: Arc<dyn PropertyEditor>) {
        self.default_editors
            .write()
            .unwrap()
            .insert(type_id, editor);
    }

    /// 查找默认属性编辑器。
    pub fn find_default_editor(&self, type_id: TypeId) -> Option<Arc<dyn PropertyEditor>> {
        self.default_editors.read().unwrap().get(&type_id).cloned()
    }

    /// 查找属性编辑器（优先自定义，其次默认）。
    ///
    /// 对应 Spring 的查找策略：先查自定义编辑器，再查默认编辑器。
    pub fn find_editor(&self, type_id: TypeId) -> Option<Arc<dyn PropertyEditor>> {
        self.find_custom_editor(type_id)
            .or_else(|| self.find_default_editor(type_id))
    }

    /// 是否已注册默认编辑器。
    pub fn has_default_editors(&self) -> bool {
        *self.default_editors_registered.read().unwrap()
    }

    /// 标记默认编辑器已注册。
    pub fn mark_default_editors_registered(&self) {
        *self.default_editors_registered.write().unwrap() = true;
    }

    /// 自定义编辑器数量。
    pub fn custom_editor_count(&self) -> usize {
        self.custom_editors.read().unwrap().len()
    }

    /// 默认编辑器数量。
    pub fn default_editor_count(&self) -> usize {
        self.default_editors.read().unwrap().len()
    }

    /// 是否包含指定类型的自定义编辑器。
    pub fn has_custom_editor_for(&self, type_id: TypeId) -> bool {
        self.custom_editors.read().unwrap().contains_key(&type_id)
    }

    /// 清空所有缓存。
    pub fn clear(&self) {
        self.custom_editors.write().unwrap().clear();
        self.default_editors.write().unwrap().clear();
        *self.default_editors_registered.write().unwrap() = false;
    }

    /// 仅清空自定义编辑器缓存。
    pub fn clear_custom_editors(&self) {
        self.custom_editors.write().unwrap().clear();
    }
}

impl Default for PropertyEditorCache {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PropertyEditorCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyEditorCache")
            .field("custom_count", &self.custom_editor_count())
            .field("default_count", &self.default_editor_count())
            .field("defaults_registered", &self.has_default_editors())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    /// 测试用简单编辑器
    #[derive(Debug)]
    struct StubEditor {
        value: String,
    }

    impl StubEditor {
        fn new(initial: &str) -> Self {
            Self {
                value: initial.to_string(),
            }
        }
    }

    impl PropertyEditor for StubEditor {
        fn target_type(&self) -> TypeId {
            TypeId::of::<String>()
        }

        fn set_as_text(
            &mut self,
            text: &str,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.value = text.to_string();
            Ok(())
        }

        fn get_as_text(&self) -> Option<String> {
            Some(self.value.clone())
        }

        fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
            if let Some(s) = value.downcast_ref::<String>() {
                self.value = s.clone();
            }
        }

        fn get_value(&self) -> Option<&dyn Any> {
            Some(&self.value)
        }

        fn get_value_type(&self) -> TypeId {
            TypeId::of::<String>()
        }
    }

    #[test]
    fn register_and_find_custom_editor() {
        let cache = PropertyEditorCache::new();
        let editor = Arc::new(StubEditor::new("test"));
        cache.register_custom_editor(TypeId::of::<String>(), editor);

        assert!(cache.has_custom_editor_for(TypeId::of::<String>()));
        assert_eq!(cache.custom_editor_count(), 1);

        let found = cache.find_custom_editor(TypeId::of::<String>());
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_as_text(), Some("test".to_string()));
    }

    #[test]
    fn find_editor_prefers_custom_over_default() {
        let cache = PropertyEditorCache::new();
        let custom = Arc::new(StubEditor::new("custom"));
        let default = Arc::new(StubEditor::new("default"));

        cache.register_default_editor(TypeId::of::<i32>(), default);
        cache.register_custom_editor(TypeId::of::<i32>(), custom);

        let found = cache.find_editor(TypeId::of::<i32>()).unwrap();
        assert_eq!(found.get_as_text(), Some("custom".to_string()));
    }

    #[test]
    fn clear_removes_all() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("a")));
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("b")));
        cache.mark_default_editors_registered();

        cache.clear();
        assert_eq!(cache.custom_editor_count(), 0);
        assert_eq!(cache.default_editor_count(), 0);
        assert!(!cache.has_default_editors());
    }

    #[test]
    fn default_editors_registration_flag() {
        let cache = PropertyEditorCache::new();
        assert!(!cache.has_default_editors());
        cache.mark_default_editors_registered();
        assert!(cache.has_default_editors());
    }

    #[test]
    fn find_editor_falls_back_to_default() {
        let cache = PropertyEditorCache::new();
        let default_editor = Arc::new(StubEditor::new("default"));
        cache.register_default_editor(TypeId::of::<i32>(), default_editor);

        // No custom editor registered, should fall back to default
        let found = cache.find_editor(TypeId::of::<i32>());
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_as_text(), Some("default".to_string()));
    }

    #[test]
    fn find_editor_returns_none_when_empty() {
        let cache = PropertyEditorCache::new();
        assert!(cache.find_editor(TypeId::of::<String>()).is_none());
    }

    #[test]
    fn find_custom_editor_not_registered() {
        let cache = PropertyEditorCache::new();
        assert!(cache.find_custom_editor(TypeId::of::<String>()).is_none());
    }

    #[test]
    fn find_default_editor_not_registered() {
        let cache = PropertyEditorCache::new();
        assert!(cache.find_default_editor(TypeId::of::<String>()).is_none());
    }

    #[test]
    fn has_custom_editor_for_false() {
        let cache = PropertyEditorCache::new();
        assert!(!cache.has_custom_editor_for(TypeId::of::<String>()));
    }

    #[test]
    fn default_editor_count() {
        let cache = PropertyEditorCache::new();
        assert_eq!(cache.default_editor_count(), 0);
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("a")));
        assert_eq!(cache.default_editor_count(), 1);
    }

    #[test]
    fn clear_custom_editors_only() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("custom")));
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
        cache.mark_default_editors_registered();

        cache.clear_custom_editors();
        assert_eq!(cache.custom_editor_count(), 0);
        assert_eq!(cache.default_editor_count(), 1);
        assert!(cache.has_default_editors());
    }

    #[test]
    fn register_multiple_custom_editors() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("s")));
        cache.register_custom_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("i")));
        cache.register_custom_editor(TypeId::of::<bool>(), Arc::new(StubEditor::new("b")));
        assert_eq!(cache.custom_editor_count(), 3);
    }

    #[test]
    fn register_overwrites_previous_custom_editor() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("first")));
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("second")));
        assert_eq!(cache.custom_editor_count(), 1);
        let found = cache.find_custom_editor(TypeId::of::<String>()).unwrap();
        assert_eq!(found.get_as_text(), Some("second".to_string()));
    }

    #[test]
    fn default_trait_works() {
        let cache = PropertyEditorCache::default();
        assert!(!cache.has_default_editors());
        assert_eq!(cache.custom_editor_count(), 0);
    }

    #[test]
    fn debug_format() {
        let cache = PropertyEditorCache::new();
        let debug = format!("{:?}", cache);
        assert!(debug.contains("PropertyEditorCache"));
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn register_default_editor_multiple() {
        let cache = PropertyEditorCache::new();
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("i32")));
        cache.register_default_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("str")));
        assert_eq!(cache.default_editor_count(), 2);
    }

    #[test]
    fn register_default_editor_overwrites() {
        let cache = PropertyEditorCache::new();
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("first")));
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("second")));
        assert_eq!(cache.default_editor_count(), 1);
        let found = cache.find_default_editor(TypeId::of::<i32>()).unwrap();
        assert_eq!(found.get_as_text(), Some("second".to_string()));
    }

    #[test]
    fn has_custom_editor_for_true() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("test")));
        assert!(cache.has_custom_editor_for(TypeId::of::<String>()));
    }

    #[test]
    fn has_custom_editor_for_false_v2() {
        let cache = PropertyEditorCache::new();
        assert!(!cache.has_custom_editor_for(TypeId::of::<String>()));
    }

    #[test]
    fn find_editor_custom_only() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("custom")));
        let found = cache.find_editor(TypeId::of::<String>()).unwrap();
        assert_eq!(found.get_as_text(), Some("custom".to_string()));
    }

    #[test]
    fn find_editor_default_only() {
        let cache = PropertyEditorCache::new();
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
        let found = cache.find_editor(TypeId::of::<i32>()).unwrap();
        assert_eq!(found.get_as_text(), Some("default".to_string()));
    }

    #[test]
    fn mark_default_editors_registered_and_check() {
        let cache = PropertyEditorCache::new();
        assert!(!cache.has_default_editors());
        cache.mark_default_editors_registered();
        assert!(cache.has_default_editors());
    }

    #[test]
    fn clear_resets_default_editors_registered() {
        let cache = PropertyEditorCache::new();
        cache.mark_default_editors_registered();
        assert!(cache.has_default_editors());
        cache.clear();
        assert!(!cache.has_default_editors());
    }

    #[test]
    fn clear_custom_editors_preserves_defaults() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("custom")));
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
        cache.mark_default_editors_registered();
        cache.clear_custom_editors();
        assert_eq!(cache.custom_editor_count(), 0);
        assert_eq!(cache.default_editor_count(), 1);
        assert!(cache.has_default_editors());
    }

    #[test]
    fn find_editor_returns_none_for_unregistered() {
        let cache = PropertyEditorCache::new();
        assert!(cache.find_editor(TypeId::of::<bool>()).is_none());
    }

    #[test]
    fn find_custom_editor_returns_none_for_unregistered() {
        let cache = PropertyEditorCache::new();
        assert!(cache.find_custom_editor(TypeId::of::<bool>()).is_none());
    }

    #[test]
    fn find_default_editor_returns_none_for_unregistered() {
        let cache = PropertyEditorCache::new();
        assert!(cache.find_default_editor(TypeId::of::<bool>()).is_none());
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[test]
    fn find_editor_custom_and_default_both_registered() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("custom")));
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
        let found = cache.find_editor(TypeId::of::<i32>()).unwrap();
        assert_eq!(found.get_as_text(), Some("custom".to_string()));
    }

    #[test]
    fn clear_and_reregister() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("first")));
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
        cache.mark_default_editors_registered();
        cache.clear();
        assert_eq!(cache.custom_editor_count(), 0);
        assert_eq!(cache.default_editor_count(), 0);
        assert!(!cache.has_default_editors());
        // Re-register
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("second")));
        assert_eq!(cache.custom_editor_count(), 1);
    }

    #[test]
    fn debug_format_contains_fields() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("test")));
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
        cache.mark_default_editors_registered();
        let debug = format!("{:?}", cache);
        assert!(debug.contains("PropertyEditorCache"));
        assert!(debug.contains("custom_count"));
        assert!(debug.contains("default_count"));
        assert!(debug.contains("defaults_registered"));
    }

    #[test]
    fn register_default_editor_overwrites_v2() {
        let cache = PropertyEditorCache::new();
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("first")));
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("second")));
        assert_eq!(cache.default_editor_count(), 1);
        let found = cache.find_default_editor(TypeId::of::<i32>()).unwrap();
        assert_eq!(found.get_as_text(), Some("second".to_string()));
    }

    #[test]
    fn clear_custom_editors_preserves_defaults_v2() {
        let cache = PropertyEditorCache::new();
        cache.register_custom_editor(TypeId::of::<String>(), Arc::new(StubEditor::new("custom")));
        cache.register_default_editor(TypeId::of::<i32>(), Arc::new(StubEditor::new("default")));
        cache.mark_default_editors_registered();
        cache.clear_custom_editors();
        assert_eq!(cache.custom_editor_count(), 0);
        assert_eq!(cache.default_editor_count(), 1);
        assert!(cache.has_default_editors());
    }
}
