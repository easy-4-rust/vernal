//! bean_definition_override_exception — 对应 Java 异常类。
use std::fmt;

/// BeanDefinitionOverrideException 异常。
#[derive(Debug, Clone)]
pub struct BeanDefinitionOverrideException {
    message: String,
}

impl BeanDefinitionOverrideException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for BeanDefinitionOverrideException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BeanDefinitionOverrideException {}
