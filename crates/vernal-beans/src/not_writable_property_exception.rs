//! not_writable_property_exception — 对应 Java 异常类。

use std::fmt;

/// NotWritablePropertyException 异常。
#[derive(Debug, Clone)]
pub struct NotWritablePropertyException {
    message: String,
}

impl NotWritablePropertyException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for NotWritablePropertyException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NotWritablePropertyException {}
