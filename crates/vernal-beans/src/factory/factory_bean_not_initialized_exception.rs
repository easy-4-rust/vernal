//! factory_bean_not_initialized_exception — 对应 Java 异常类。
use std::fmt;

/// FactoryBeanNotInitializedException 异常。
#[derive(Debug, Clone)]
pub struct FactoryBeanNotInitializedException {
    message: String,
}

impl FactoryBeanNotInitializedException {
    /// 创建一个新的实例。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
    /// 获取消息。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for FactoryBeanNotInitializedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FactoryBeanNotInitializedException {}
