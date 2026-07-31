//! URLEditor — Spring 风格的 URL 属性编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.URLEditor`。
//!
//! 将字符串转换为 URL 字符串。验证 URL 格式的合法性，
//! 支持 http、https、ftp 等协议。
//!
//! ## 设计说明
//!
//! 在 vernal 中，URL 以字符串形式存储，解析时验证格式合法性。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// URL 属性编辑器。
///
/// 对应 Spring 的 `URLEditor`。
///
/// 将字符串转换为合法的 URL 字符串。
#[derive(Debug, Default)]
pub struct URLEditor {
    value: Option<String>,
}

impl URLEditor {
    /// 创建 URL 属性编辑器。
    pub fn new() -> Self {
        Self { value: None }
    }

    /// 验证 URL 格式是否合法。
    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("ftp://")
            || url.starts_with("file://")
    }
}

impl PropertyEditor for URLEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.value = None;
            return Ok(());
        }

        if Self::is_valid_url(trimmed) {
            self.value = Some(trimmed.to_string());
            Ok(())
        } else {
            Err(format!("Invalid URL: '{}'. Must start with http://, https://, ftp://, or file://", trimmed).into())
        }
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
    fn valid_http_url() {
        let mut editor = URLEditor::new();
        editor.set_as_text("http://example.com").unwrap();
        assert_eq!(editor.get_as_text(), Some("http://example.com".to_string()));
    }

    #[test]
    fn valid_https_url() {
        let mut editor = URLEditor::new();
        editor.set_as_text("https://example.com/path?query=1").unwrap();
        assert!(editor.get_as_text().is_some());
    }

    #[test]
    fn invalid_url_returns_error() {
        let mut editor = URLEditor::new();
        assert!(editor.set_as_text("not-a-url").is_err());
    }

    #[test]
    fn empty_string_gives_none() {
        let mut editor = URLEditor::new();
        editor.set_as_text("http://example.com").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn ftp_url_is_valid() {
        let mut editor = URLEditor::new();
        editor.set_as_text("ftp://files.example.com/pub").unwrap();
        assert!(editor.get_as_text().is_some());
    }

    #[test]
    fn is_valid_url_check() {
        assert!(URLEditor::is_valid_url("http://example.com"));
        assert!(URLEditor::is_valid_url("https://example.com"));
        assert!(URLEditor::is_valid_url("ftp://example.com"));
        assert!(URLEditor::is_valid_url("file:///tmp/test"));
        assert!(!URLEditor::is_valid_url("not-a-url"));
    }

    #[test]
    fn target_type_is_string() {
        let editor = URLEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn get_value_type_is_string() {
        let editor = URLEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn set_value_with_string() {
        let mut editor = URLEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new("https://example.com".to_string());
        editor.set_value(val);
        assert_eq!(editor.get_as_text(), Some("https://example.com".to_string()));
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = URLEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn get_value_returns_ref() {
        let mut editor = URLEditor::new();
        editor.set_as_text("http://test.com").unwrap();
        let val = editor.get_value().unwrap();
        assert!(val.is::<String>());
        assert_eq!(val.downcast_ref::<String>().unwrap(), "http://test.com");
    }

    #[test]
    fn get_value_none_when_empty() {
        let editor = URLEditor::new();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn default_creates_empty_editor() {
        let editor = URLEditor::default();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn file_url() {
        let mut editor = URLEditor::new();
        editor.set_as_text("file:///home/user/data.txt").unwrap();
        assert_eq!(editor.get_as_text(), Some("file:///home/user/data.txt".to_string()));
    }

    #[test]
    fn https_with_port() {
        let mut editor = URLEditor::new();
        editor.set_as_text("https://example.com:8080/path").unwrap();
        assert_eq!(editor.get_as_text(), Some("https://example.com:8080/path".to_string()));
    }

    #[test]
    fn whitespace_trimmed() {
        let mut editor = URLEditor::new();
        editor.set_as_text("  https://example.com  ").unwrap();
        assert_eq!(editor.get_as_text(), Some("https://example.com".to_string()));
    }

    #[test]
    fn invalid_schemes() {
        let mut editor = URLEditor::new();
        assert!(editor.set_as_text("ssh://server").is_err());
        assert!(editor.set_as_text("mailto:user@test.com").is_err());
        assert!(editor.set_as_text("ws://socket").is_err());
    }

    #[test]
    fn whitespace_only_gives_none() {
        let mut editor = URLEditor::new();
        editor.set_as_text("   ").unwrap();
        assert!(editor.get_as_text().is_none());
    }
}
