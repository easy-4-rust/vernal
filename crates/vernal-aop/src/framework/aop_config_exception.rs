//! AOP 配置异常。
//!
//! 对应 spring-aop `org.springframework.aop.framework.AopConfigException`。

/// AOP 配置异常。
///
/// 对应 spring-aop `AopConfigException`。
///
/// 当 AOP 配置无效时抛出。
#[derive(Debug)]
pub struct AopConfigException {
    message: String,
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl AopConfigException {
    /// 创建新的 AOP 配置异常。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
        }
    }

    /// 创建带原因的 AOP 配置异常。
    pub fn with_cause(
        message: impl Into<String>,
        cause: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self {
            message: message.into(),
            cause: Some(cause),
        }
    }

    /// 获取错误消息。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 获取原因。
    pub fn cause(&self) -> Option<&(dyn std::error::Error + Send + Sync)> {
        self.cause.as_ref().map(|c| c.as_ref())
    }
}

impl std::fmt::Display for AopConfigException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AopConfigException: {}", self.message)
    }
}

impl std::error::Error for AopConfigException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|c| c.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aop_config_exception_display() {
        let err = AopConfigException::new("invalid configuration");
        assert_eq!(
            format!("{}", err),
            "AopConfigException: invalid configuration"
        );
    }

    #[test]
    fn aop_config_exception_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad input");
        let err = AopConfigException::with_cause("wrapper error", Box::new(cause));
        assert!(err.cause().is_some());
    }

    #[test]
    fn aop_config_exception_message() {
        let err = AopConfigException::new("test message");
        assert_eq!(err.message(), "test message");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn aop_config_exception_new() {
        let err = AopConfigException::new("test error");
        assert_eq!(err.message(), "test error");
        assert!(err.cause().is_none());
    }

    #[test]
    fn aop_config_exception_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AopConfigException::with_cause("wrapper error", Box::new(cause));
        assert_eq!(err.message(), "wrapper error");
        assert!(err.cause().is_some());
    }

    #[test]
    fn aop_config_exception_display() {
        let err = AopConfigException::new("test error");
        assert_eq!(format!("{}", err), "AopConfigException: test error");
    }

    #[test]
    fn aop_config_exception_display_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AopConfigException::with_cause("wrapper", Box::new(cause));
        assert!(format!("{}", err).contains("wrapper"));
    }

    #[test]
    fn aop_config_exception_source_none() {
        let err = AopConfigException::new("test");
        let error: &dyn std::error::Error = &err;
        assert!(error.source().is_none());
    }

    #[test]
    fn aop_config_exception_source_some() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AopConfigException::with_cause("wrapper", Box::new(cause));
        let error: &dyn std::error::Error = &err;
        assert!(error.source().is_some());
    }

    #[test]
    fn aop_config_exception_debug() {
        let err = AopConfigException::new("test");
        let debug = format!("{:?}", err);
        assert!(debug.contains("test"));
    }

    #[test]
    fn aop_config_exception_error_trait() {
        let err = AopConfigException::new("test");
        let _: &dyn std::error::Error = &err;
    }
}
