//! bean_instantiation_exception — 对应 Java 异常类。

use std::fmt;

/// BeanInstantiationException 异常。
#[derive(Debug, Clone)]
pub struct BeanInstantiationException {
    message: String,
}

impl BeanInstantiationException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for BeanInstantiationException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BeanInstantiationException {}
