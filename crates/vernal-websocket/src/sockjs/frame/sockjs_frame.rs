//! 对应 Java 类：org.springframework.web.socket.sockjs.frame.SockJsFrame
//!
//! 表示一个 SockJS 帧，提供工厂方法创建 open/heartbeat/message/close 帧。
//! 对标 Spring `SockJsFrame` 的全部构造与解析行为，包括对单字符内容的默认填充。

use crate::sockjs::frame::SockJsFrameType;
use crate::sockjs::frame::sockjs_message_codec::SockJsMessageCodec;

/// SockJS 帧内容错误。
#[derive(Debug)]
pub struct SockJsFrameContentError(String);

impl std::fmt::Display for SockJsFrameContentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SockJsFrameContentError {}

impl SockJsFrameContentError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

/// SockJS 帧。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SockJsFrame {
    frame_type: SockJsFrameType,
    content: String,
}

impl SockJsFrame {
    /// 从原始帧内容创建，按 Spring 行为推断 type 并补齐默认内容。
    ///
    /// # Errors
    ///
    /// 当内容为空或不是合法 SockJS 帧时返回错误。
    pub fn new(content: impl Into<String>) -> Result<Self, SockJsFrameContentError> {
        let content = content.into();
        if content.is_empty() {
            return Err(SockJsFrameContentError::new("Content must not be empty"));
        }
        let first = content.chars().next().expect("checked non-empty");
        let (frame_type, normalized) = match first {
            'o' if content == "o" => (SockJsFrameType::Open, content),
            'h' if content == "h" => (SockJsFrameType::Heartbeat, content),
            'a' => (
                SockJsFrameType::Message,
                if content.len() > 1 {
                    content
                } else {
                    "a[]".to_owned()
                },
            ),
            'm' => (
                SockJsFrameType::Message,
                if content.len() > 1 {
                    content
                } else {
                    "null".to_owned()
                },
            ),
            'c' => (
                SockJsFrameType::Close,
                if content.len() > 1 {
                    content
                } else {
                    "c[]".to_owned()
                },
            ),
            _ => {
                return Err(SockJsFrameContentError::new(format!(
                    "Unexpected SockJS frame type in content \"{content}\""
                )));
            }
        };
        Ok(Self {
            frame_type,
            content: normalized,
        })
    }

    /// 创建 open 帧 `o`。
    #[must_use]
    pub fn open_frame() -> Self {
        Self {
            frame_type: SockJsFrameType::Open,
            content: "o".to_owned(),
        }
    }

    /// 创建 heartbeat 帧 `h`。
    #[must_use]
    pub fn heartbeat_frame() -> Self {
        Self {
            frame_type: SockJsFrameType::Heartbeat,
            content: "h".to_owned(),
        }
    }

    /// 创建 message 帧：用 codec 编码 messages。
    #[must_use]
    pub fn message_frame(codec: &dyn SockJsMessageCodec, messages: &[&str]) -> Self {
        let encoded = codec.encode(messages);
        Self::new(encoded).expect("codec 输出必为合法 SockJS 帧")
    }

    /// 创建 close 帧 `c[code,"reason"]`。
    #[must_use]
    pub fn close_frame(code: u32, reason: Option<&str>) -> Self {
        let reason = reason.unwrap_or("");
        let content = format!("c[{code},\"{reason}\"]");
        Self::new(content).expect("close 帧格式合法")
    }

    /// 创建 Spring `closeFrameGoAway` 等价帧：`c[3000,"Go away!"]`。
    #[must_use]
    pub fn close_frame_go_away() -> Self {
        Self::close_frame(3000, Some("Go away!"))
    }

    /// 创建 Spring `closeFrameAnotherConnectionOpen` 等价帧。
    #[must_use]
    pub fn close_frame_another_connection_open() -> Self {
        Self::close_frame(2010, Some("Another connection still open"))
    }

    /// 返回帧类型。
    #[must_use]
    pub const fn frame_type(&self) -> SockJsFrameType {
        self.frame_type
    }

    /// 返回帧内容。
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// 返回帧内容的字节（UTF-8）。
    #[must_use]
    pub fn content_bytes(&self) -> Vec<u8> {
        self.content.as_bytes().to_vec()
    }

    /// 返回 open/heartbeat 帧以外的帧数据（去掉首字符）。
    #[must_use]
    pub fn frame_data(&self) -> Option<&str> {
        match self.frame_type {
            SockJsFrameType::Open | SockJsFrameType::Heartbeat => None,
            _ => self.content.get(1..),
        }
    }
}

impl std::fmt::Display for SockJsFrame {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const MAX_LENGTH: usize = 80;
        let length = self.content.len().min(MAX_LENGTH);
        formatter.write_str("SockJsFrame content='")?;
        for c in self.content.chars().take(length) {
            match c {
                '\n' => formatter.write_str("\\n")?,
                '\r' => formatter.write_str("\\r")?,
                _ => write!(formatter, "{c}")?,
            }
        }
        if length < self.content.len() {
            formatter.write_str("...(truncated)")?;
        }
        formatter.write_str("'")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_and_heartbeat_frames_match_spring_constants() {
        assert_eq!(SockJsFrame::open_frame().content(), "o");
        assert_eq!(SockJsFrame::heartbeat_frame().content(), "h");
    }

    #[test]
    fn message_frame_with_empty_messages_normalizes_to_a_brackets() {
        let frame = SockJsFrame::new("a").unwrap();
        assert_eq!(frame.content(), "a[]");
        assert_eq!(frame.frame_type(), SockJsFrameType::Message);
    }

    #[test]
    fn close_frame_go_away_matches_spring_value() {
        let frame = SockJsFrame::close_frame_go_away();
        assert_eq!(frame.content(), "c[3000,\"Go away!\"]");
    }

    #[test]
    fn close_frame_another_connection_matches_spring_value() {
        let frame = SockJsFrame::close_frame_another_connection_open();
        assert_eq!(frame.content(), "c[2010,\"Another connection still open\"]");
    }

    #[test]
    fn frame_data_strips_first_char_for_message_and_close() {
        let message = SockJsFrame::new("a[\"m\"]").unwrap();
        assert_eq!(message.frame_data(), Some("[\"m\"]"));
        let open = SockJsFrame::open_frame();
        assert!(open.frame_data().is_none());
    }

    #[test]
    fn rejects_empty_and_invalid_content() {
        assert!(SockJsFrame::new("").is_err());
        assert!(SockJsFrame::new("x").is_err());
    }
}
