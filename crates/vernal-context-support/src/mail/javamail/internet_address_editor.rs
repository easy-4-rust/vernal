//! 邮箱地址属性编辑器 — 对标 `InternetAddressEditor`。

/// 邮箱地址属性编辑器。
///
/// 对标 Spring 的 `InternetAddressEditor`，将字符串值转换为邮箱地址。
pub struct InternetAddressEditor {
    address: Option<String>,
}

impl InternetAddressEditor {
    /// 创建属性编辑器。
    pub fn new() -> Self {
        Self { address: None }
    }

    /// 从字符串设置邮箱地址。
    pub fn set_as_text(&mut self, text: &str) -> Result<(), String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            self.address = None;
        } else {
            // 简单的邮箱格式验证
            if !trimmed.contains('@') {
                return Err(format!("无效的邮箱地址：{trimmed}"));
            }
            self.address = Some(trimmed.to_string());
        }
        Ok(())
    }

    /// 获取邮箱地址的字符串形式。
    pub fn get_as_text(&self) -> String {
        self.address.clone().unwrap_or_default()
    }

    /// 获取邮箱地址。
    pub fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }
}

impl Default for InternetAddressEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_valid_address() {
        let mut editor = InternetAddressEditor::new();
        editor.set_as_text("user@example.com").unwrap();
        assert_eq!(editor.address(), Some("user@example.com"));
        assert_eq!(editor.get_as_text(), "user@example.com");
    }

    #[test]
    fn test_set_empty_address() {
        let mut editor = InternetAddressEditor::new();
        editor.set_as_text("user@example.com").unwrap();
        editor.set_as_text("").unwrap();
        assert!(editor.address().is_none());
        assert_eq!(editor.get_as_text(), "");
    }

    #[test]
    fn test_set_blank_address() {
        let mut editor = InternetAddressEditor::new();
        editor.set_as_text("  ").unwrap();
        assert!(editor.address().is_none());
    }

    #[test]
    fn test_set_invalid_address() {
        let mut editor = InternetAddressEditor::new();
        assert!(editor.set_as_text("not-an-email").is_err());
    }

    #[test]
    fn test_get_as_text_null() {
        let editor = InternetAddressEditor::new();
        assert_eq!(editor.get_as_text(), "");
    }
}
