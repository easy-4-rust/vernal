//! TimeZoneEditor — Spring 风格的时区属性编辑器。
//!
//! 对应 Java 类：`org.springframework.beans.propertyeditors.TimeZoneEditor`。
//!
//! 将字符串转换为时区标识符（用 `String` 表示）。
//! 支持标准的 IANA 时区名称，如 "America/New_York"、"Asia/Shanghai" 等。
//!
//! ## 设计说明
//!
//! 在 vernal 中，时区以字符串标识符存储，不依赖外部时区库。

use std::any::Any;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// 时区属性编辑器。
///
/// 对应 Spring 的 `TimeZoneEditor`。
///
/// 将字符串转换为时区标识符。
#[derive(Debug, Default)]
pub struct TimeZoneEditor {
    value: Option<String>,
}

impl TimeZoneEditor {
    /// 创建时区属性编辑器。
    pub fn new() -> Self {
        Self { value: None }
    }

    /// 获取已知的有效时区列表。
    pub fn known_timezones() -> &'static [&'static str] {
        &[
            "UTC", "GMT",
            "America/New_York", "America/Chicago", "America/Denver", "America/Los_Angeles",
            "Europe/London", "Europe/Paris", "Europe/Berlin", "Europe/Moscow",
            "Asia/Tokyo", "Asia/Shanghai", "Asia/Kolkata", "Asia/Dubai",
            "Australia/Sydney", "Australia/Melbourne",
            "Pacific/Auckland", "Pacific/Honolulu",
        ]
    }
}

impl PropertyEditor for TimeZoneEditor {
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }

    fn set_as_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.value = None;
            return Ok(());
        }

        // 基本格式验证：时区标识符应包含 "/" 或是 "UTC"/"GMT"
        if trimmed.contains('/') || trimmed == "UTC" || trimmed == "GMT" {
            self.value = Some(trimmed.to_string());
            Ok(())
        } else {
            Err(format!("Invalid timezone format: '{}'. Expected IANA format like 'America/New_York'", trimmed).into())
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
    fn set_valid_timezone() {
        let mut editor = TimeZoneEditor::new();
        editor.set_as_text("Asia/Shanghai").unwrap();
        assert_eq!(editor.get_as_text(), Some("Asia/Shanghai".to_string()));
    }

    #[test]
    fn set_utc() {
        let mut editor = TimeZoneEditor::new();
        editor.set_as_text("UTC").unwrap();
        assert_eq!(editor.get_as_text(), Some("UTC".to_string()));
    }

    #[test]
    fn invalid_timezone_returns_error() {
        let mut editor = TimeZoneEditor::new();
        assert!(editor.set_as_text("InvalidTZ").is_err());
    }

    #[test]
    fn empty_string_gives_none() {
        let mut editor = TimeZoneEditor::new();
        editor.set_as_text("America/New_York").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn known_timezones_not_empty() {
        assert!(!TimeZoneEditor::known_timezones().is_empty());
    }

    #[test]
    fn set_gmt() {
        let mut editor = TimeZoneEditor::new();
        editor.set_as_text("GMT").unwrap();
        assert_eq!(editor.get_as_text(), Some("GMT".to_string()));
    }

    #[test]
    fn set_with_whitespace() {
        let mut editor = TimeZoneEditor::new();
        editor.set_as_text("  UTC  ").unwrap();
        assert_eq!(editor.get_as_text(), Some("UTC".to_string()));
    }

    #[test]
    fn target_type_is_string() {
        let editor = TimeZoneEditor::new();
        assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn get_value_type_is_string() {
        let editor = TimeZoneEditor::new();
        assert_eq!(editor.get_value_type(), std::any::TypeId::of::<String>());
    }

    #[test]
    fn set_value_with_string() {
        let mut editor = TimeZoneEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new("Asia/Tokyo".to_string());
        editor.set_value(val);
        assert_eq!(editor.get_as_text(), Some("Asia/Tokyo".to_string()));
    }

    #[test]
    fn set_value_with_wrong_type_ignored() {
        let mut editor = TimeZoneEditor::new();
        let val: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42i32);
        editor.set_value(val);
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn get_value_returns_ref() {
        let mut editor = TimeZoneEditor::new();
        editor.set_as_text("Europe/London").unwrap();
        let val = editor.get_value().unwrap();
        assert!(val.is::<String>());
        assert_eq!(val.downcast_ref::<String>().unwrap(), "Europe/London");
    }

    #[test]
    fn get_value_none_when_empty() {
        let editor = TimeZoneEditor::new();
        assert!(editor.get_value().is_none());
    }

    #[test]
    fn known_timezones_contains_common_zones() {
        let zones = TimeZoneEditor::known_timezones();
        assert!(zones.contains(&"UTC"));
        assert!(zones.contains(&"GMT"));
        assert!(zones.contains(&"America/New_York"));
        assert!(zones.contains(&"Asia/Shanghai"));
        assert!(zones.contains(&"Europe/London"));
    }

    #[test]
    fn whitespace_only_sets_none() {
        let mut editor = TimeZoneEditor::new();
        editor.set_as_text("   ").unwrap();
        assert!(editor.get_as_text().is_none());
    }

    #[test]
    fn various_valid_timezones() {
        let mut editor = TimeZoneEditor::new();
        for tz in &["America/New_York", "Europe/Paris", "Asia/Tokyo", "Australia/Sydney"] {
            editor.set_as_text(tz).unwrap();
            assert_eq!(editor.get_as_text(), Some(tz.to_string()));
        }
    }

    #[test]
    fn default_creates_empty_editor() {
        let editor = TimeZoneEditor::default();
        assert!(editor.get_as_text().is_none());
    }
}
