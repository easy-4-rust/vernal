//! scope_not_active_exception — 对应 Java 异常类。
use std::fmt;

/// ScopeNotActiveException 异常。
#[derive(Debug, Clone)]
pub struct ScopeNotActiveException {
    message: String,
}

impl ScopeNotActiveException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for ScopeNotActiveException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ScopeNotActiveException {}
