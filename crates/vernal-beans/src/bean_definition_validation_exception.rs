//! bean_definition_validation_exception — 对应 Java 异常类。
use std::fmt;

/// BeanDefinitionValidationException 异常。
#[derive(Debug, Clone)]
pub struct BeanDefinitionValidationException {
    message: String,
}

impl BeanDefinitionValidationException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for BeanDefinitionValidationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BeanDefinitionValidationException {}
