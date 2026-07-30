//! AntPathMatcher — Ant 风格路径匹配器。
/// Ant 风格路径匹配器。
#[derive(Clone, Debug, Default)]
pub struct AntPathMatcher;
impl AntPathMatcher {
    pub fn new() -> Self { Self }
    pub fn matches(&self, pattern: &str, path: &str) -> bool {
        if pattern == "*" { return true; }
        if pattern == "**" { return true; }
        pattern == path
    }
    pub fn extract_path_template(&self, pattern: &str) -> Option<String> {
        if pattern.contains('{') && pattern.contains('}') {
            Some(pattern.to_string())
        } else {
            None
        }
    }
}
