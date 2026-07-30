//! MessagingTemplate — 消息模板。
use crate::messaging::Message;

/// 消息模板。
#[derive(Clone, Debug, Default)]
pub struct MessagingTemplate;
impl MessagingTemplate {
    pub fn new() -> Self { Self }
    pub fn send(&self, _destination: &str, _message: &dyn Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    pub fn receive(&self, _destination: &str) -> Result<Option<Box<dyn Message>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
}
