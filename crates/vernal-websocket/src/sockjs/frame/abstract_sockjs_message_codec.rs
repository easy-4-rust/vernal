//! 对应 Java 类：org.springframework.web.socket.sockjs.frame.AbstractSockJsMessageCodec
//!
//! 提供基于 JSON 引号转义 + SockJS 特殊字符 Unicode 转义的默认 encode 实现。
//! 对标 Spring `AbstractSockJsMessageCodec.encode`，包括 SockJS 协议测试套件中
//! `escapable_by_server` 的字符范围。

use crate::sockjs::frame::sockjs_message_codec::{SockJsCodecError, SockJsMessageCodec};

/// 抽象 codec，提供 encode 默认实现；decode 委派给子类。
pub trait AbstractSockJsMessageCodec: SockJsMessageCodec {
    /// 子类实现：对内容应用标准 JSON 引号转义。
    fn apply_json_quoting(&self, content: &str) -> String;

    /// 默认 encode：`a["q1","q2"]`，按 SockJS 协议再做特殊字符转义。
    fn encode_messages(&self, messages: &[&str]) -> String {
        let mut buffer = String::from("a[");
        for (index, message) in messages.iter().enumerate() {
            buffer.push('"');
            let quoted = self.apply_json_quoting(message);
            buffer.push_str(&escape_sockjs_special_chars(&quoted));
            buffer.push('"');
            if index + 1 < messages.len() {
                buffer.push(',');
            }
        }
        buffer.push(']');
        buffer
    }
}

/// 对 SockJS 协议规定的特殊字符（U+0000-001F, U+200C-200F, U+2028-202F, U+2060-206F,
/// U+FFF0+, U+D800-DFFF）做 `\uXXXX` 转义。
pub fn escape_sockjs_special_chars(content: &str) -> String {
    let mut result = String::new();
    for c in content.chars() {
        if is_sockjs_special_char(c) {
            result.push_str(&format!("\\u{:04x}", c as u32));
        } else {
            result.push(c);
        }
    }
    result
}

fn is_sockjs_special_char(ch: char) -> bool {
    let code = ch as u32;
    code <= 0x001F
        || (0x200C..=0x200F).contains(&code)
        || (0x2028..=0x202F).contains(&code)
        || (0x2060..=0x206F).contains(&code)
        || code >= 0xFFF0
        || (0xD800..=0xDFFF).contains(&code)
}

/// 把 JSON codec 适配为 `AbstractSockJsMessageCodec`。
pub trait JsonQuotingCodec: AbstractSockJsMessageCodec {
    /// 解码 content 为消息列表。
    ///
    /// # Errors
    ///
    /// 解析失败时返回错误。
    fn decode_messages(&self, content: &str) -> Result<Vec<String>, SockJsCodecError>;
}

impl<T: AbstractSockJsMessageCodec + ?Sized> SockJsMessageCodec for &T {
    fn encode(&self, messages: &[&str]) -> String {
        (**self).encode_messages(messages)
    }
    fn decode(&self, content: &str) -> Result<Vec<String>, SockJsCodecError> {
        // 委派给具体 codec 自身的 decode。
        // 这里需要 T: SockJsMessageCodec，但 &T 已经是；故通过 trait 对象无法调用，
        // 留给具体子类直接实现 SockJsMessageCodec::decode。
        Err(SockJsCodecError(format!(
            "AbstractSockJsMessageCodec::decode 未实现：{content}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PassThroughCodec;
    impl AbstractSockJsMessageCodec for PassThroughCodec {
        fn apply_json_quoting(&self, content: &str) -> String {
            // 测试用：仅做 JSON 标准转义（" 和 \）
            content.replace('\\', "\\\\").replace('"', "\\\"")
        }
    }
    impl SockJsMessageCodec for PassThroughCodec {
        fn encode(&self, messages: &[&str]) -> String {
            self.encode_messages(messages)
        }
        fn decode(&self, _content: &str) -> Result<Vec<String>, SockJsCodecError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn encode_wraps_messages_in_a_array() {
        let codec = PassThroughCodec;
        assert_eq!(codec.encode(&[]), "a[]");
        assert_eq!(codec.encode(&["hello"]), "a[\"hello\"]");
        assert_eq!(codec.encode(&["a", "b"]), "a[\"a\",\"b\"]");
    }

    #[test]
    fn escape_replaces_control_chars_with_unicode_sequences() {
        assert_eq!(escape_sockjs_special_chars("\u{0001}"), "\\u0001");
        assert_eq!(escape_sockjs_special_chars("\u{2028}"), "\\u2028");
        assert_eq!(escape_sockjs_special_chars("A"), "A");
    }

    #[test]
    fn json_quoting_escapes_backslash_and_quote() {
        let codec = PassThroughCodec;
        assert_eq!(codec.encode(&["a\"b\\c"]), "a[\"a\\\"b\\\\c\"]");
    }
}
