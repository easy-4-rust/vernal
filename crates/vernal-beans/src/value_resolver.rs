//! ValueResolver — 值解析器 trait。
/// 值解析器 trait。
pub trait ValueResolver: Send + Sync {
    fn resolve_value(&self, value: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
