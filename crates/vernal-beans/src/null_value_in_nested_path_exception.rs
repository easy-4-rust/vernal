//! null_value_in_nested_path_exception — 对应 Java 异常类。

use std::fmt;

/// NullValueInNestedPathException 异常。
#[derive(Debug, Clone)]
pub struct NullValueInNestedPathException {
    message: String,
}

impl NullValueInNestedPathException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for NullValueInNestedPathException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NullValueInNestedPathException {}
