//! aot_processing_exception — 对应 Java 异常类。
use std::fmt;

/// AotProcessingException 异常。
#[derive(Debug, Clone)]
pub struct AotProcessingException {
    message: String,
}

impl AotProcessingException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for AotProcessingException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AotProcessingException {}
