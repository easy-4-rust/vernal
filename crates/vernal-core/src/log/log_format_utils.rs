//! 日志格式工具。
//!
//! 对标 Spring `org.springframework.core.log.LogFormatUtils`。

/// 日志格式工具（静态辅助函数）。
///
/// 对应 Java: org.springframework.core.log.LogFormatUtils
pub struct LogFormatUtils;

impl LogFormatUtils {
    /// 格式化值并截断到指定长度（对标 Spring `formatValue` 的 limit 语义）。
    #[must_use]
    pub fn format_value(value: &str, limit: usize) -> String {
        if value.chars().count() <= limit {
            value.to_string()
        } else {
            let truncated: String = value.chars().take(limit).collect();
            format!("{truncated}... (truncated)")
        }
    }

    /// 判断值是否应被敏感信息脱敏（对标 Spring `shouldMask`）。
    #[must_use]
    pub fn should_mask(value: &str) -> bool {
        value.to_lowercase().contains("password")
            || value.to_lowercase().contains("secret")
            || value.to_lowercase().contains("token")
    }

    /// 脱敏输出（对标 Spring 日志脱敏惯例）。
    #[must_use]
    pub fn mask(value: &str) -> String {
        if Self::should_mask(value) {
            "******".to_string()
        } else {
            value.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_long_values() {
        // A 类（合同对齐）：对标 Spring formatValue limit
        assert_eq!(LogFormatUtils::format_value("hello world", 5), "hello... (truncated)");
        assert_eq!(LogFormatUtils::format_value("short", 100), "short");
    }

    #[test]
    fn masks_sensitive_keys() {
        // B 类（边界行为）：对标 Spring 脱敏
        assert!(LogFormatUtils::should_mask("my.password"));
        assert_eq!(LogFormatUtils::mask("my.password=123"), "******");
        assert_eq!(LogFormatUtils::mask("username=alice"), "username=alice");
    }
}
