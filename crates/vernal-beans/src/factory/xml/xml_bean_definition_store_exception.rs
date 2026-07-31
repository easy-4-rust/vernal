//! xml_bean_definition_store_exception — 对应 Java 异常类。
use std::fmt;

/// XmlBeanDefinitionStoreException 异常。
#[derive(Debug, Clone)]
pub struct XmlBeanDefinitionStoreException {
    message: String,
}

impl XmlBeanDefinitionStoreException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for XmlBeanDefinitionStoreException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for XmlBeanDefinitionStoreException {}
