//! TypeFilter — 类型过滤器。
use std::fmt;

/// 类型过滤器 trait。
pub trait TypeFilter: Send + Sync + fmt::Debug {
    fn match_type(&self, type_name: &str) -> bool;
}

/// 正则类型过滤器。
#[derive(Clone, Debug)]
pub struct RegexTypeFilter {
    pub pattern: String,
}
impl RegexTypeFilter {
    pub fn new(pattern: impl Into<String>) -> Self { Self { pattern: pattern.into() } }
}
impl TypeFilter for RegexTypeFilter {
    fn match_type(&self, type_name: &str) -> bool {
        type_name.contains(&self.pattern)
    }
}
