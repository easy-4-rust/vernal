//! FileEditor — Spring 风格的文件路径编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.FileEditor`。
//! 将字符串转换为文件路径。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的文件路径编辑器。
pub struct FileEditor {
    value: Option<String>,
}

impl FileEditor {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for FileEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for FileEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        self.value = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
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
    fn get_value_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_editor_has_no_value() {
        let editor = FileEditor::new();
        assert!(editor.get_as_text().is_none());
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn default_creates_empty_editor() {
        let editor = FileEditor::default();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn set_valid_path() {
        let mut editor = FileEditor::new();
        editor.set_as_text("/tmp/test.txt").unwrap();
        assert_eq!(editor.get_as_text(), Some("/tmp/test.txt".to_string()));
    }

    #[test]
    fn set_with_whitespace_trimmed() {
        let mut editor = FileEditor::new();
        editor.set_as_text("  /var/log/app.log  ").unwrap();
        assert_eq!(editor.get_as_text(), Some("/var/log/app.log".to_string()));
    }

    #[test]
    fn empty_string_sets_none() {
        let mut editor = FileEditor::new();
        editor.set_as_text("/tmp/file").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn whitespace_only_sets_none() {
        let mut editor = FileEditor::new();
        editor.set_as_text("   ").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn target_type_is_string() {
        let editor = FileEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn get_value_type_is_string() {
        let editor = FileEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn set_value_with_string() {
        let mut editor = FileEditor::new();
        let value: Arc<dyn Any + Send + Sync> = Arc::new("/path/to/file".to_string());
        editor.set_value(value);
        assert_eq!(editor.get_as_text(), Some("/path/to/file".to_string()));
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = FileEditor::new();
        let value: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(value);
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn get_value_returns_ref() {
        let mut editor = FileEditor::new();
        editor.set_as_text("/test/path").unwrap();
        let val = editor.get_value().unwrap();
        assert!(val.is::<String>());
        assert_eq!(val.downcast_ref::<String>().unwrap(), "/test/path");
    }

    #[test]
    fn get_value_when_empty() {
        let editor = FileEditor::new();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn relative_path() {
        let mut editor = FileEditor::new();
        editor.set_as_text("./relative/path.txt").unwrap();
        assert_eq!(editor.get_as_text(), Some("./relative/path.txt".to_string()));
    }

    #[test]
    fn windows_style_path() {
        let mut editor = FileEditor::new();
        editor.set_as_text("C:\\Users\\test\\file.txt").unwrap();
        assert_eq!(editor.get_as_text(), Some("C:\\Users\\test\\file.txt".to_string()));
    }
}
