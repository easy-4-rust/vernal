//! MessageSource trait — Spring 风格的消息源。
use std::fmt;

/// 消息源 trait。
pub trait MessageSource: Send + Sync + fmt::Debug {
    fn get_message(&self, code: &str, args: &[&str], default_message: Option<&str>) -> String;
}
