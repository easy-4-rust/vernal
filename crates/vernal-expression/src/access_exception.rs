//! 访问异常。
//!
//! 对标 Spring 的 `AccessException`。

/// 访问异常。
///
/// 由属性访问器在访问失败时抛出。
/// 对标 Spring 的 `org.springframework.expression.AccessException`。
#[derive(Debug, Clone)]
pub struct AccessException {
    /// 错误消息
    message: String,
}

impl AccessException {
    /// 创建访问异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// 获取错误消息。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for AccessException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "访问错误: {}", self.message)
    }
}

impl std::error::Error for AccessException {}
