//! DelegatingMessageSource — 委托消息源。
use crate::message_source::MessageSource;

/// 委托消息源。
#[derive(Clone, Debug, Default)]
pub struct DelegatingMessageSource;
impl DelegatingMessageSource {
    pub fn new() -> Self { Self }
}
impl MessageSource for DelegatingMessageSource {
    fn get_message(&self, _code: &str, _args: &[&str], default: Option<&str>) -> String {
        default.unwrap_or_default().to_string()
    }
}
