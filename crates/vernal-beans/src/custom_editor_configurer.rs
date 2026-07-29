//! CustomEditorConfigurer — Spring 风格的自定义编辑器配置器。
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use crate::property_editor::PropertyEditor;

/// Spring 风格的自定义编辑器配置器。
pub struct CustomEditorConfigurer {
    editors: HashMap<TypeId, Box<dyn PropertyEditor>>,
}

impl CustomEditorConfigurer {
    pub fn new() -> Self { Self { editors: HashMap::new() } }
    pub fn register_custom_editor(&mut self, type_id: TypeId, editor: Box<dyn PropertyEditor>) {
        self.editors.insert(type_id, editor);
    }
    pub fn get_custom_editor(&self, type_id: TypeId) -> Option<&(dyn PropertyEditor + 'static)> {
        self.editors.get(&type_id).map(|e| e.as_ref())
    }
    pub fn has_custom_editor(&self, type_id: TypeId) -> bool { self.editors.contains_key(&type_id) }
    pub fn editor_count(&self) -> usize { self.editors.len() }
}

impl Default for CustomEditorConfigurer {
    fn default() -> Self { Self::new() }
}

impl fmt::Debug for CustomEditorConfigurer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomEditorConfigurer")
            .field("editor_count", &self.editors.len())
            .finish()
    }
}
