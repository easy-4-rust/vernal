//! PropertyEditorRegistrar — 属性编辑器注册器。
/// 属性编辑器注册器 trait。
pub trait PropertyEditorRegistrar: Send + Sync {
    fn register_custom_editors(&self, registry: &mut dyn crate::property_editor_registry::PropertyEditorRegistry);
}
