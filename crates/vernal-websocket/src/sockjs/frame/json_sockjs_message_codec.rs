//! 对应 Java 类：org.springframework.web.socket.sockjs.frame.Jackson2SockJsMessageCodec
//! 与 JacksonJsonSockJsMessageCodec（合并 serde_json 实现）
//!
//! Spring 用 Jackson 2.x 或 tools.jackson 3.x；Vernal 使用 `serde_json` 等价实现，
//! 提供 JSON 引号转义与帧编解码。

use crate::sockjs::frame::abstract_sockjs_message_codec::AbstractSockJsMessageCodec;
use crate::sockjs::frame::sockjs_message_codec::{SockJsCodecError, SockJsMessageCodec};

/// 基于 serde_json 的 SockJS message codec。
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonSockJsMessageCodec;

impl JsonSockJsMessageCodec {
    /// 创建 codec。
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl AbstractSockJsMessageCodec for JsonSockJsMessageCodec {
    fn apply_json_quoting(&self, content: &str) -> String {
        // serde_json::to_string 返回带外层引号的 JSON 字符串；
        // 这里剥掉外层引号，由 `encode_messages` 在外层补回（对标 Spring
        // `char[] applyJsonQuoting` 不含外层引号的语义）。
        let quoted = serde_json::to_string(content).unwrap_or_else(|_| String::from("\"\""));
        // 仅移除首尾各一个引号；中间已转义的引号保留。
        if quoted.len() >= 2 && quoted.starts_with('"') && quoted.ends_with('"') {
            quoted[1..quoted.len() - 1].to_owned()
        } else {
            quoted
        }
    }
}

impl SockJsMessageCodec for JsonSockJsMessageCodec {
    fn encode(&self, messages: &[&str]) -> String {
        self.encode_messages(messages)
    }

    fn decode(&self, content: &str) -> Result<Vec<String>, SockJsCodecError> {
        if content.is_empty() {
            return Ok(Vec::new());
        }
        let payload = content.strip_prefix('a').unwrap_or(content);
        let parsed: Vec<String> =
            serde_json::from_str(payload).map_err(|err| SockJsCodecError(err.to_string()))?;
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_single_message() {
        let codec = JsonSockJsMessageCodec::new();
        let encoded = codec.encode(&["hello"]);
        assert_eq!(encoded, "a[\"hello\"]");
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, vec!["hello".to_string()]);
    }

    #[test]
    fn round_trip_multiple_messages_preserves_order() {
        let codec = JsonSockJsMessageCodec::new();
        let encoded = codec.encode(&["first", "second"]);
        assert_eq!(encoded, "a[\"first\",\"second\"]");
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, vec!["first".to_string(), "second".to_string()]);
    }

    #[test]
    fn escapes_special_characters_via_sockjs_rules() {
        let codec = JsonSockJsMessageCodec::new();
        let encoded = codec.encode(&["line\nbreak"]);
        assert!(encoded.contains("\\u000a") || encoded.contains("\\n"));
    }
}
