//! 路径匹配器 trait。
//!
//! 对标 Spring `org.springframework.util.PathMatcher`。

use std::collections::HashMap;

/// 路径匹配器 trait。
///
/// 对标 Spring `PathMatcher` 接口。
pub trait PathMatcher: Send + Sync {
    /// 判断给定字符串是否包含模式字符(`*` / `?` / `{var}`)。
    ///
    /// 对标 Spring `PathMatcher.isPattern(String)`。
    fn is_pattern(&self, path: &str) -> bool;

    /// 完整匹配路径。
    ///
    /// 对标 Spring `PathMatcher.match(String, String)`。
    fn matches(&self, pattern: &str, path: &str) -> bool;

    /// 仅匹配前缀。
    ///
    /// 对标 Spring `PathMatcher.matchStart(String, String)`。
    fn matches_start(&self, pattern: &str, path: &str) -> bool;

    /// 提取模式中匹配的实际路径(去除通配符部分)。
    ///
    /// 对标 Spring `PathMatcher.extractPathWithinPattern(String, String)`。
    fn extract_path_within_pattern(&self, pattern: &str, path: &str) -> String;

    /// 提取 URI 模板变量(`{var}` 形式)。
    ///
    /// 对标 Spring `PathMatcher.extractUriTemplateVariables(String, String)`。
    fn extract_uri_template_variables(&self, pattern: &str, path: &str) -> HashMap<String, String>;

    /// 合并两个模式。
    ///
    /// 对标 Spring `PathMatcher.combine(String, String)`。
    fn combine(&self, pattern1: &str, pattern2: &str) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::AntPathMatcher;

    #[test]
    fn trait_object_works() {
        let matcher: Box<dyn PathMatcher> = Box::new(AntPathMatcher::new());
        assert!(matcher.matches("/api/*", "/api/users"));
        assert!(!matcher.matches("/api/*", "/api/users/123"));
    }

    #[test]
    fn is_pattern_detects_wildcards() {
        let matcher = AntPathMatcher::new();
        assert!(!matcher.is_pattern("/api/users"));
        assert!(matcher.is_pattern("/api/*"));
        assert!(matcher.is_pattern("/api/**"));
        assert!(matcher.is_pattern("/api/{id}"));
    }

    #[test]
    fn extract_uri_template_variables() {
        let matcher = AntPathMatcher::new();
        let vars = matcher.extract_uri_template_variables("/api/{id}", "/api/123");
        assert_eq!(vars.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn combine_basic() {
        let matcher = AntPathMatcher::new();
        assert_eq!(matcher.combine("/api", "users"), "/api/users");
    }
}
