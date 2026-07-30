//! invalid_property_exception — 对应 Java 异常类。

use std::fmt;

/// InvalidPropertyException 异常。
#[derive(Debug, Clone)]
pub struct InvalidPropertyException {
    message: String,
}

impl InvalidPropertyException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for InvalidPropertyException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for InvalidPropertyException {}
