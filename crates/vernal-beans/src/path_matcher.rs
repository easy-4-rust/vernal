//! PathMatcher — 路径匹配器 trait。
/// 路径匹配器 trait。
pub trait PathMatcher: Send + Sync {
    fn matches(&self, pattern: &str, path: &str) -> bool;
    fn extract_path_template(&self, pattern: &str) -> Option<String>;
}
