//! fatal_bean_exception — 对应 Java 异常类。

use std::fmt;

/// FatalBeanException 异常。
#[derive(Debug, Clone)]
pub struct FatalBeanException {
    message: String,
}

impl FatalBeanException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
    pub fn message(&self) -> &str { &self.message }
}

impl fmt::Display for FatalBeanException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FatalBeanException {}
