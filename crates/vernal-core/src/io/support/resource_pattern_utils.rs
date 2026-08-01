//! 资源模式工具。
//!
//! 对标 Spring `org.springframework.core.io.support.ResourcePatternUtils`。

/// 资源模式工具（静态辅助函数）。
///
/// 对应 Java: org.springframework.core.io.support.ResourcePatternUtils
pub struct ResourcePatternUtils;

impl ResourcePatternUtils {
    /// 判断位置是否为 URL（`classpath:` 之外带协议的形态）。
    #[must_use]
    pub fn is_url(location: &str) -> bool {
        location.contains("://") || location.starts_with("classpath:")
    }

    /// 判断位置是否包含模式字符（`*` 或 `?`）。
    #[must_use]
    pub fn is_pattern(location: &str) -> bool {
        location.contains('*') || location.contains('?') || location.contains('{')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_url_locations() {
        // A 类（合同对齐）：对标 Spring `isUrl`
        assert!(ResourcePatternUtils::is_url("file:///tmp/x"));
        assert!(ResourcePatternUtils::is_url("classpath:x"));
        assert!(!ResourcePatternUtils::is_url("/plain/path"));
    }

    #[test]
    fn detects_pattern_locations() {
        // B 类（边界行为）：对标 Spring 模式判定
        assert!(ResourcePatternUtils::is_pattern("classpath*:*.xml"));
        assert!(ResourcePatternUtils::is_pattern("/x?y"));
        assert!(!ResourcePatternUtils::is_pattern("/plain/path"));
    }
}
