//! UUIDEditor — Spring 风格的 UUID 编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.UUIDEditor`。
//!
//! 将字符串转换为 `std::uuid::Uuid`。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的 UUID 编辑器。
///
/// 对应 Spring 的 `UUIDEditor`。
pub struct UUIDEditor {
    value: Option<String>,
}

impl UUIDEditor {
    /// 创建 UUID 编辑器。
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Default for UUIDEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyEditor for UUIDEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.value = None;
            return Ok(());
        }
        // 验证 UUID 格式（8-4-4-4-12）
        let parts: Vec<&str> = trimmed.split('-').collect();
        if parts.len() != 5 {
            return Err(format!("Invalid UUID format: {}", text).into());
        }
        if parts[0].len() != 8
            || parts[1].len() != 4
            || parts[2].len() != 4
            || parts[3].len() != 4
            || parts[4].len() != 12
        {
            return Err(format!("Invalid UUID format: {}", text).into());
        }
        // 验证每个字符都是十六进制
        for part in &parts {
            if !part.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(format!("Invalid UUID format: {}", text).into());
            }
        }
        self.value = Some(trimmed.to_string());
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
        let editor = UUIDEditor::new();
        assert!(editor.get_as_text().is_none());
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn default_creates_empty_editor() {
        let editor = UUIDEditor::default();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn valid_uuid() {
        let mut editor = UUIDEditor::new();
        editor
            .set_as_text("550e8400-e29b-41d4-a716-446655440000")
            .unwrap();
        assert_eq!(
            editor.get_as_text(),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn valid_uuid_uppercase() {
        let mut editor = UUIDEditor::new();
        editor
            .set_as_text("550E8400-E29B-41D4-A716-446655440000")
            .unwrap();
        assert_eq!(
            editor.get_as_text(),
            Some("550E8400-E29B-41D4-A716-446655440000".to_string())
        );
    }

    #[test]
    fn valid_uuid_with_whitespace() {
        let mut editor = UUIDEditor::new();
        editor
            .set_as_text("  550e8400-e29b-41d4-a716-446655440000  ")
            .unwrap();
        assert_eq!(
            editor.get_as_text(),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn empty_string_sets_none() {
        let mut editor = UUIDEditor::new();
        editor
            .set_as_text("550e8400-e29b-41d4-a716-446655440000")
            .unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn invalid_uuid_wrong_segment_count() {
        let mut editor = UUIDEditor::new();
        assert!(editor.set_as_text("550e8400-e29b-41d4").is_err());
    }

    #[test]
    fn invalid_uuid_wrong_segment_length() {
        let mut editor = UUIDEditor::new();
        assert!(
            editor
                .set_as_text("550e84-e29b-41d4-a716-446655440000")
                .is_err()
        );
    }

    #[test]
    fn invalid_uuid_non_hex() {
        let mut editor = UUIDEditor::new();
        assert!(
            editor
                .set_as_text("ZZZZZZZZ-e29b-41d4-a716-446655440000")
                .is_err()
        );
    }

    #[test]
    fn invalid_uuid_no_dashes() {
        let mut editor = UUIDEditor::new();
        assert!(
            editor
                .set_as_text("550e8400e29b41d4a716446655440000")
                .is_err()
        );
    }

    #[test]
    fn target_type_is_string() {
        let editor = UUIDEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn get_value_type_is_string() {
        let editor = UUIDEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn set_value_with_string() {
        let mut editor = UUIDEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> =
            Arc::new("550e8400-e29b-41d4-a716-446655440000".to_string());
        editor.set_value(val);
        assert_eq!(
            editor.get_as_text(),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = UUIDEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn get_value_returns_ref() {
        let mut editor = UUIDEditor::new();
        editor
            .set_as_text("550e8400-e29b-41d4-a716-446655440000")
            .unwrap();
        let val = editor.get_value().unwrap();
        assert!(val.is::<String>());
    }

    #[test]
    fn get_value_none_when_empty() {
        let editor = UUIDEditor::new();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn all_zeros_uuid() {
        let mut editor = UUIDEditor::new();
        editor
            .set_as_text("00000000-0000-0000-0000-000000000000")
            .unwrap();
        assert_eq!(
            editor.get_as_text(),
            Some("00000000-0000-0000-0000-000000000000".to_string())
        );
    }

    #[test]
    fn all_fs_uuid() {
        let mut editor = UUIDEditor::new();
        editor
            .set_as_text("ffffffff-ffff-ffff-ffff-ffffffffffff")
            .unwrap();
        assert_eq!(
            editor.get_as_text(),
            Some("ffffffff-ffff-ffff-ffff-ffffffffffff".to_string())
        );
    }

    #[test]
    fn whitespace_only_sets_none() {
        let mut editor = UUIDEditor::new();
        editor.set_as_text("   ").unwrap();
        assert!(editor.get_as_text().is_none());
    }
}
