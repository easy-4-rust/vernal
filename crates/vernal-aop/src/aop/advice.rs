//! 通知接口。
//!
//! 对应 spring-aop `org.aopalliance.aop.Advice`。
//! 所有通知的标记接口。

/// 通知标记接口。
///
/// 对应 aopalliance `Advice`。
///
/// 所有 AOP 通知都应实现此接口。这是一个标记接口，不定义任何方法。
pub trait Advice: Send + Sync + 'static {
    /// 获取通知类型名。
    fn advice_type(&self) -> &str;
}

/// 通知错误。
///
/// 对应 aopalliance `AspectException`。
#[derive(Debug)]
pub struct AspectException {
    message: String,
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl AspectException {
    /// 创建新的切面异常。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
        }
    }

    /// 创建带原因的切面异常。
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

impl std::fmt::Display for AspectException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AspectException: {}", self.message)
    }
}

impl std::error::Error for AspectException {
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
    fn aspect_exception_display() {
        let err = AspectException::new("test error");
        assert_eq!(format!("{}", err), "AspectException: test error");
    }

    #[test]
    fn aspect_exception_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AspectException::with_cause("wrapper error", Box::new(cause));
        assert!(err.cause().is_some());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    struct TestAdvice {
        advice_type: String,
    }

    impl TestAdvice {
        fn new(advice_type: &str) -> Self {
            Self {
                advice_type: advice_type.to_string(),
            }
        }
    }

    impl Advice for TestAdvice {
        fn advice_type(&self) -> &str {
            &self.advice_type
        }
    }

    #[test]
    fn advice_trait_object() {
        let advice: Box<dyn Advice> = Box::new(TestAdvice::new("test"));
        assert_eq!(advice.advice_type(), "test");
    }

    #[test]
    fn aspect_exception_new() {
        let err = AspectException::new("test error");
        assert_eq!(err.message(), "test error");
        assert!(err.cause().is_none());
    }

    #[test]
    fn aspect_exception_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AspectException::with_cause("wrapper error", Box::new(cause));
        assert_eq!(err.message(), "wrapper error");
        assert!(err.cause().is_some());
    }

    #[test]
    fn aspect_exception_display() {
        let err = AspectException::new("test error");
        assert_eq!(format!("{}", err), "AspectException: test error");
    }

    #[test]
    fn aspect_exception_display_with_cause() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AspectException::with_cause("wrapper", Box::new(cause));
        assert!(format!("{}", err).contains("wrapper"));
    }

    #[test]
    fn aspect_exception_cause_none() {
        let err = AspectException::new("test");
        assert!(err.cause().is_none());
    }

    #[test]
    fn aspect_exception_cause_some() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AspectException::with_cause("wrapper", Box::new(cause));
        assert!(err.cause().is_some());
    }

    #[test]
    fn aspect_exception_error_source_none() {
        let err = AspectException::new("test");
        let error: &dyn std::error::Error = &err;
        assert!(error.source().is_none());
    }

    #[test]
    fn aspect_exception_error_source_some() {
        let cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = AspectException::with_cause("wrapper", Box::new(cause));
        let error: &dyn std::error::Error = &err;
        assert!(error.source().is_some());
    }

    #[test]
    fn aspect_exception_debug() {
        let err = AspectException::new("test");
        let debug = format!("{:?}", err);
        assert!(debug.contains("test"));
    }

    #[test]
    fn aspect_exception_error_trait() {
        let err = AspectException::new("test");
        let _: &dyn std::error::Error = &err;
    }
}
