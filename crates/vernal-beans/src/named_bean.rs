//! NamedBean — 命名 Bean trait。
/// 命名 Bean trait。
pub trait NamedBean: Send + Sync {
    fn get_bean_name(&self) -> &str;
}
