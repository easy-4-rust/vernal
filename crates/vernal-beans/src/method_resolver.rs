//! MethodResolver — 方法解析器。
/// 方法解析器 trait。
pub trait MethodResolver: Send + Sync {
    fn resolve(&self, method_name: &str) -> Option<String>;
}
