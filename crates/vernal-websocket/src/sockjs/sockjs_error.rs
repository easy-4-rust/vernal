//! 对应 Java 类：org.springframework.web.socket.sockjs.SockJsException
//!
//! SockJS HTTP 请求处理异常基类。

use std::fmt;

/// SockJS 异常基类。
#[derive(Debug)]
pub struct SockJsError {
    /// 异常消息。
    pub message: String,
    /// 关联的 SockJS session id（可能为 None）。
    pub session_id: Option<String>,
    /// 可选的根因。
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl SockJsError {
    /// 创建异常。
    #[must_use]
    pub fn new(
        message: impl Into<String>,
        session_id: Option<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self {
            message: message.into(),
            session_id,
            source,
        }
    }
}

impl fmt::Display for SockJsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.session_id {
            Some(id) => write!(formatter, "{} (session={})", self.message, id),
            None => formatter.write_str(&self.message),
        }
    }
}

impl std::error::Error for SockJsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
