//! HierarchicalMessageSource — 层级消息源。
use crate::message_source::MessageSource;

/// 层级消息源。
#[derive(Clone, Debug, Default)]
pub struct HierarchicalMessageSource;
impl HierarchicalMessageSource {
    pub fn new() -> Self { Self }
}
impl MessageSource for HierarchicalMessageSource {
    fn get_message(&self, _code: &str, _args: &[&str], default: Option<&str>) -> String {
        default.unwrap_or_default().to_string()
    }
}
