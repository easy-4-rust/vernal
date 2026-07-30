//! Messaging — 消息系统基础。
use std::any::Any;
use std::fmt;

/// 消息 trait。
pub trait Message: Send + Sync + fmt::Debug {
    fn get_payload(&self) -> &dyn Any;
    fn get_headers(&self) -> std::collections::HashMap<String, String>;
}

/// 消息处理器 trait。
pub trait MessageHandler: Send + Sync + fmt::Debug {
    fn handle_message(&self, message: &dyn Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
