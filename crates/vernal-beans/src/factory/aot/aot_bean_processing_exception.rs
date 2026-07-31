//! aot_bean_processing_exception — 对应 Java 异常类。
use std::fmt;

/// AotBeanProcessingException 异常。
#[derive(Debug, Clone)]
pub struct AotBeanProcessingException {
    message: String,
}

impl AotBeanProcessingException {
    /// 创建一个新的实例。
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    /// 获取消息。
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for AotBeanProcessingException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AotBeanProcessingException {}
