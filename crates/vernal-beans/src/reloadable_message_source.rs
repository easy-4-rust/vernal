//! ReloadableMessageSource — 可重载消息源。
use crate::message_source::MessageSource;

/// 可重载消息源。
#[derive(Clone, Debug, Default)]
pub struct ReloadableMessageSource;
impl ReloadableMessageSource {
    pub fn new() -> Self { Self }
    pub fn reload(&self) {}
}
impl MessageSource for ReloadableMessageSource {
    fn get_message(&self, _code: &str, _args: &[&str], default: Option<&str>) -> String {
        default.unwrap_or_default().to_string()
    }
}
