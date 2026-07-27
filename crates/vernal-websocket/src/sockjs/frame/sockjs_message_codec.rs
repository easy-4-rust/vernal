//! 对应 Java 类：org.springframework.web.socket.sockjs.frame.SockJsMessageCodec
//!
//! 把消息数组编码/解码为 SockJS message 帧（JSON 数组）。

/// SockJS 消息 codec SPI。
pub trait SockJsMessageCodec: Send + Sync {
    /// 编码消息数组为 SockJS message 帧（如 `a["m1","m2"]`）。
    fn encode(&self, messages: &[&str]) -> String;

    /// 解码 SockJS message 帧内容为消息数组；空内容返回空 Vec。
    ///
    /// # Errors
    ///
    /// 当内容无法解析时返回错误。
    fn decode(&self, content: &str) -> Result<Vec<String>, SockJsCodecError>;
}

/// SockJS codec 错误。
#[derive(Debug)]
pub struct SockJsCodecError(pub String);

impl std::fmt::Display for SockJsCodecError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SockJsCodecError {}
