//! bean_is_abstract_exception — 对应 Java 异常类。
use std::fmt;

/// BeanIsAbstractException 异常。
#[derive(Debug, Clone)]
pub struct BeanIsAbstractException {
    message: String,
}

impl BeanIsAbstractException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for BeanIsAbstractException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BeanIsAbstractException {}
