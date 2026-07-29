//! StringValueResolver trait — 字符串值解析器。
use std::fmt;

/// 字符串值解析器 trait。
pub trait StringValueResolver: Send + Sync + fmt::Debug {
    fn resolve_string_value(&self, value: &str) -> String;
}

/// 恒等字符串值解析器。
#[derive(Clone, Debug, Default)]
pub struct IdentityStringValueResolver;
impl StringValueResolver for IdentityStringValueResolver {
    fn resolve_string_value(&self, value: &str) -> String { value.to_string() }
}
