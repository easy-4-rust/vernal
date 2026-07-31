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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_editor::PropertyEditor;
    use std::any::Any;
    use std::sync::Arc;

    struct StubEditor {
        text: Option<String>,
    }

    impl StubEditor {
        fn new() -> Self { Self { text: None } }
    }

    impl PropertyEditor for StubEditor {
        fn target_type(&self) -> TypeId { TypeId::of::<String>() }
        fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.text = Some(text.to_string());
            Ok(())
        }
        fn get_as_text(&self) -> Option<String> { self.text.clone() }
        fn set_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
            if let Some(s) = value.downcast_ref::<String>() {
                self.text = Some(s.clone());
            }
        }
        fn get_value(&self) -> Option<&dyn Any> { self.text.as_ref().map(|v| v as &dyn Any) }
        fn get_value_type(&self) -> TypeId { TypeId::of::<String>() }
    }

    #[test]
    fn new_is_empty() {
        let configurer = CustomEditorConfigurer::new();
        assert_eq!(configurer.editor_count(), 0);
    }

    #[test]
    fn default_is_empty() {
        let configurer = CustomEditorConfigurer::default();
        assert_eq!(configurer.editor_count(), 0);
    }

    #[test]
    fn register_and_retrieve_editor() {
        let mut configurer = CustomEditorConfigurer::new();
        configurer.register_custom_editor(TypeId::of::<i32>(), Box::new(StubEditor::new()));
        assert_eq!(configurer.editor_count(), 1);
        assert!(configurer.has_custom_editor(TypeId::of::<i32>()));
        assert!(!configurer.has_custom_editor(TypeId::of::<String>()));
    }

    #[test]
    fn get_custom_editor_returns_none_when_not_registered() {
        let configurer = CustomEditorConfigurer::new();
        assert!(configurer.get_custom_editor(TypeId::of::<i32>()).is_none());
    }

    #[test]
    fn get_custom_editor_returns_editor() {
        let mut configurer = CustomEditorConfigurer::new();
        let mut editor = StubEditor::new();
        editor.set_as_text("test_value").unwrap();
        configurer.register_custom_editor(TypeId::of::<String>(), Box::new(editor));

        let found = configurer.get_custom_editor(TypeId::of::<String>()).unwrap();
        assert_eq!(found.get_as_text(), Some("test_value".to_string()));
    }

    #[test]
    fn register_overwrites_previous() {
        let mut configurer = CustomEditorConfigurer::new();
        let mut editor1 = StubEditor::new();
        editor1.set_as_text("first").unwrap();
        configurer.register_custom_editor(TypeId::of::<String>(), Box::new(editor1));

        let mut editor2 = StubEditor::new();
        editor2.set_as_text("second").unwrap();
        configurer.register_custom_editor(TypeId::of::<String>(), Box::new(editor2));

        assert_eq!(configurer.editor_count(), 1);
        let found = configurer.get_custom_editor(TypeId::of::<String>()).unwrap();
        assert_eq!(found.get_as_text(), Some("second".to_string()));
    }

    #[test]
    fn register_multiple_types() {
        let mut configurer = CustomEditorConfigurer::new();
        configurer.register_custom_editor(TypeId::of::<i32>(), Box::new(StubEditor::new()));
        configurer.register_custom_editor(TypeId::of::<String>(), Box::new(StubEditor::new()));
        configurer.register_custom_editor(TypeId::of::<bool>(), Box::new(StubEditor::new()));
        assert_eq!(configurer.editor_count(), 3);
    }

    #[test]
    fn has_custom_editor_false_for_unregistered() {
        let configurer = CustomEditorConfigurer::new();
        assert!(!configurer.has_custom_editor(TypeId::of::<i32>()));
    }

    #[test]
    fn debug_format() {
        let mut configurer = CustomEditorConfigurer::new();
        configurer.register_custom_editor(TypeId::of::<i32>(), Box::new(StubEditor::new()));
        let debug_str = format!("{:?}", configurer);
        assert!(debug_str.contains("CustomEditorConfigurer"));
        assert!(debug_str.contains("editor_count"));
    }
}
