//! UnsatisfiedDependencyException — 对应 Spring `org.springframework.beans.factory.UnsatisfiedDependencyException`。
//!
//! 依赖注入失败时抛出的异常。

use std::fmt;

/// 依赖注入失败时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.UnsatisfiedDependencyException`。
#[derive(Debug)]
pub struct UnsatisfiedDependencyException {
    bean_name: String,
    injection_point: String,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl UnsatisfiedDependencyException {
    /// 创建一个新的 UnsatisfiedDependencyException。
    pub fn new(
        bean_name: impl Into<String>,
        injection_point: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            bean_name: bean_name.into(),
            injection_point: injection_point.into(),
            message: message.into(),
            source: None,
        }
    }

    /// 创建一个带有原因的 UnsatisfiedDependencyException。
    pub fn with_cause(
        bean_name: impl Into<String>,
        injection_point: impl Into<String>,
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            bean_name: bean_name.into(),
            injection_point: injection_point.into(),
            message: message.into(),
            source: Some(Box::new(cause)),
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取注入点。
    pub fn injection_point(&self) -> &str {
        &self.injection_point
    }
}

impl fmt::Display for UnsatisfiedDependencyException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unsatisfied dependency expressed through field '{}': {}",
            self.injection_point, self.message
        )
    }
}

impl std::error::Error for UnsatisfiedDependencyException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_new() {
        let e = UnsatisfiedDependencyException::new("myBean", "userService", "No qualifying bean");
        assert_eq!(e.bean_name(), "myBean");
        assert_eq!(e.injection_point(), "userService");
    }

    #[test]
    fn test_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::Other, "inner");
        let e = UnsatisfiedDependencyException::with_cause("myBean", "field", "msg", cause);
        assert!(e.source().is_some());
    }

    #[test]
    fn test_display() {
        let e = UnsatisfiedDependencyException::new("myBean", "userService", "not found");
        assert!(format!("{}", e).contains("userService"));
    }
}
