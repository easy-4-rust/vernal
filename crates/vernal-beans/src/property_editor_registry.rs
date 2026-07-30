//! PropertyEditorRegistry — 属性编辑器注册表。
use std::any::TypeId;

/// 属性编辑器注册表 trait。
pub trait PropertyEditorRegistry: Send + Sync {
    fn register_custom_editor(&mut self, required_type: TypeId, editor: Box<dyn crate::property_editor::PropertyEditor>);
    fn find_custom_editor(&self, required_type: TypeId) -> Option<&dyn crate::property_editor::PropertyEditor>;
}
