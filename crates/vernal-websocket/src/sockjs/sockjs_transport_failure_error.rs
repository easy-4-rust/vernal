//! 对应 Java 类：org.springframework.web.socket.sockjs.SockJsTransportFailureException
//!
//! SockJS 实现内部发生严重失败（如 I/O 写失败）；通常会关闭 session。

use crate::sockjs::sockjs_error::SockJsError;

/// 构造 SockJs transport 失败。
#[must_use]
pub fn transport_failure(
    message: impl Into<String>,
    session_id: Option<String>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
) -> SockJsError {
    SockJsError::new(message, session_id, source)
}
