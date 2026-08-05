//! ScopeNotActiveException — Scope 未激活异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ScopeNotActiveException`。
//!
//! 在 Spring 中，当请求的 Scope（如 request、session）在当前上下文中
//! 未激活时抛出此异常。例如在非 Web 上下文中访问 request scope 的 Bean。

use std::fmt;

/// Scope 未激活异常。
///
/// 对应 Spring 的 `ScopeNotActiveException`。
///
/// 当请求的 Scope 在当前上下文中不可用时抛出。
#[derive(Debug, Clone)]
pub struct ScopeNotActiveException {
    scope_name: String,
    message: String,
}

impl ScopeNotActiveException {
    /// 创建新的异常。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            scope_name: String::new(),
            message: message.into(),
        }
    }

    /// 创建带 Scope 名称的异常。
    pub fn with_scope_name(scope_name: impl Into<String>) -> Self {
        let name = scope_name.into();
        Self {
            message: format!("Scope '{}' is not active in the current context", name),
            scope_name: name,
        }
    }

    /// 获取错误消息。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 获取 Scope 名称。
    pub fn scope_name(&self) -> &str {
        &self.scope_name
    }
}

impl fmt::Display for ScopeNotActiveException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ScopeNotActiveException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_exception_has_message() {
        let e = ScopeNotActiveException::new("scope not found");
        assert_eq!(e.message(), "scope not found");
    }

    #[test]
    fn with_scope_name() {
        let e = ScopeNotActiveException::with_scope_name("request");
        assert_eq!(e.scope_name(), "request");
        assert!(e.message().contains("request"));
        assert!(e.message().contains("not active"));
    }

    #[test]
    fn display_and_error_trait() {
        let e = ScopeNotActiveException::with_scope_name("session");
        assert_eq!(format!("{}", e), e.message());
        let err: &dyn std::error::Error = &e;
        assert!(!err.to_string().is_empty());
    }
}
