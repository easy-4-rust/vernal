//! MessageSourceAware — 消息源感知 trait。
use crate::message_source::MessageSource;

/// 消息源感知 trait。
pub trait MessageSourceAware: Send + Sync {
    fn set_message_source(&mut self, source: Box<dyn MessageSource>);
}
