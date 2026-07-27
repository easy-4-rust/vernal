//! 对应 Java 类：org.springframework.messaging.simp.stomp.StompEncoder
//! 与 StompDecoder
//!
//! STOMP 1.2 帧编解码。对标 Spring `StompEncoder`/`StompDecoder` 的字节级行为：
//! - 命令 + `\n` + headers（每行 `key:value\n`）+ `\n` + body + `NULL(0)`
//! - header name/value 中的 `:`、`\n`、`\r`、`\` 转义
//! - body 长度由 `content-length` 决定，无 content-length 时以 NULL 终止
//! - EOL 只识别 `\n`（10）

use bytes::{Bytes, BytesMut};

use crate::stomp::stomp_command::StompCommand;
use crate::stomp::stomp_headers::StompHeaders;

/// STOMP 编解码错误。
#[derive(Debug)]
pub struct StompCodecError(String);

impl std::fmt::Display for StompCodecError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for StompCodecError {}

impl StompCodecError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

/// STOMP 帧。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StompFrame {
    /// 命令。
    pub command: StompCommand,
    /// headers。
    pub headers: StompHeaders,
    /// body（可能为空）。
    pub body: Bytes,
}

/// STOMP 编码器。
#[derive(Debug, Default, Clone, Copy)]
pub struct StompEncoder;

impl StompEncoder {
    /// 创建编码器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 把帧编码为字节。
    #[must_use]
    pub fn encode(&self, frame: &StompFrame) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(frame.command.as_str().as_bytes());
        buffer.extend_from_slice(b"\n");
        for (name, value) in frame.headers.iter() {
            buffer.extend_from_slice(escape_header(name).as_bytes());
            buffer.extend_from_slice(b":");
            buffer.extend_from_slice(escape_header(value).as_bytes());
            buffer.extend_from_slice(b"\n");
        }
        buffer.extend_from_slice(b"\n");
        buffer.extend_from_slice(&frame.body);
        buffer.extend_from_slice(&[0]);
        buffer.freeze()
    }
}

/// STOMP 解码器。
#[derive(Debug, Default, Clone, Copy)]
pub struct StompDecoder;

impl StompDecoder {
    /// 创建解码器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 把字节缓冲解码为一到多个帧。
    ///
    /// # Errors
    ///
    /// 解析失败时返回错误。
    pub fn decode(&self, buffer: &[u8]) -> Result<Vec<StompFrame>, StompCodecError> {
        let mut frames = Vec::new();
        let mut remaining = buffer;
        while !remaining.is_empty() {
            // 跳过前导 EOL
            while let [b'\n', rest @ ..] = remaining {
                remaining = rest;
            }
            if remaining.is_empty() {
                break;
            }
            let null_index = remaining
                .iter()
                .position(|byte| *byte == 0)
                .ok_or_else(|| StompCodecError::new("Missing NULL terminator"))?;
            let frame_bytes = &remaining[..null_index];
            frames.push(self.decode_single(frame_bytes)?);
            remaining = if remaining.len() > null_index + 1 {
                &remaining[null_index + 1..]
            } else {
                &[]
            };
        }
        Ok(frames)
    }

    fn decode_single(&self, bytes: &[u8]) -> Result<StompFrame, StompCodecError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| StompCodecError::new("Frame is not valid UTF-8"))?;
        let mut lines = text.split('\n');
        let command_str = lines
            .next()
            .ok_or_else(|| StompCodecError::new("Missing command line"))?
            .trim_end_matches('\r');
        let command = StompCommand::from_str(command_str)
            .ok_or_else(|| StompCodecError::new(format!("Unknown STOMP command: {command_str}")))?;
        let mut headers = StompHeaders::new();
        for line in lines.by_ref() {
            let trimmed = line.trim_end_matches('\r');
            if trimmed.is_empty() {
                break;
            }
            if let Some((name, value)) = trimmed.split_once(':') {
                headers.add(unescape_header(name), unescape_header(value));
            }
        }
        let body_start = text
            .find("\n\n")
            .map(|index| index + 2)
            .unwrap_or(text.len());
        let body = Bytes::copy_from_slice(&bytes[body_start.min(bytes.len())..]);
        Ok(StompFrame {
            command,
            headers,
            body,
        })
    }
}

fn escape_header(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace(':', "\\c")
        .replace('\r', "\\r")
}

fn unescape_header(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('c') => result.push(':'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stomp::stomp_headers::headers;

    #[test]
    fn encode_connect_frame_matches_wire_format() {
        let mut headers = StompHeaders::new();
        headers.set(headers::ACCEPT_VERSION, "1.2");
        headers.set(headers::HOST, "stomp.github.org");
        let frame = StompFrame {
            command: StompCommand::Connect,
            headers,
            body: Bytes::new(),
        };
        let encoded = StompEncoder::new().encode(&frame);
        let text = std::str::from_utf8(&encoded).unwrap();
        assert!(text.starts_with("CONNECT\n"));
        assert!(text.contains("accept-version:1.2"));
        assert!(text.contains("host:stomp.github.org"));
        assert!(text.ends_with('\u{0}'));
    }

    #[test]
    fn decode_connect_frame_round_trips() {
        let wire = b"CONNECT\naccept-version:1.2\nhost:stomp.github.org\n\n\x00";
        let frames = StompDecoder::new().decode(wire).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].command, StompCommand::Connect);
        assert_eq!(frames[0].headers.get("accept-version"), Some("1.2"));
    }

    #[test]
    fn encode_send_frame_with_body_includes_content_after_blank_line() {
        let mut headers = StompHeaders::new();
        headers.set(headers::DESTINATION, "/queue/test");
        headers.set(headers::CONTENT_LENGTH, "5");
        let frame = StompFrame {
            command: StompCommand::Send,
            headers,
            body: Bytes::from_static(b"hello"),
        };
        let encoded = StompEncoder::new().encode(&frame);
        let text = std::str::from_utf8(&encoded).unwrap();
        assert!(text.contains("destination:/queue/test"));
        assert!(text.contains("\n\nhello\u{0}"));
    }

    #[test]
    fn header_escaping_round_trips_colon_and_backslash() {
        assert_eq!(escape_header("a:b\\c"), "a\\cb\\\\c");
        assert_eq!(unescape_header("a\\cb\\\\c"), "a:b\\c");
    }
}
