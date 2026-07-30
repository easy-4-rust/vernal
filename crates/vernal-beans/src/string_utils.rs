//! StringUtils — 字符串工具。
/// 字符串工具。
pub struct StringUtils;
impl StringUtils {
    pub fn has_length(str: &str) -> bool { !str.is_empty() }
    pub fn has_text(str: &str) -> bool { !str.trim().is_empty() }
    pub fn starts_with(str: &str, prefix: &str) -> bool { str.starts_with(prefix) }
    pub fn ends_with(str: &str, suffix: &str) -> bool { str.ends_with(suffix) }
    pub fn trim_whitespace(str: &str) -> &str { str.trim() }
}
