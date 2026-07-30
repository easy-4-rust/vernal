//! not_readable_property_exception — 对应 Java 异常类。

use std::fmt;

/// NotReadablePropertyException 异常。
#[derive(Debug, Clone)]
pub struct NotReadablePropertyException {
    message: String,
}

impl NotReadablePropertyException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for NotReadablePropertyException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NotReadablePropertyException {}
