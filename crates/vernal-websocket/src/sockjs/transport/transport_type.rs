//! 对应 Java 类：org.springframework.web.socket.sockjs.transport.TransportType
//!
//! SockJS 传输类型。对标 Spring `TransportType` 的所有属性与方法。

use http::Method;
use std::collections::HashMap;
use std::sync::OnceLock;

/// SockJS 传输类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportType {
    /// WebSocket 传输。
    WebSocket,
    /// XHR POST 接收端（用于 send）。
    XhrSend,
    /// XHR 流式传输。
    XhrStreaming,
    /// XHR 轮询传输。
    Xhr,
    /// EventSource 传输。
    EventSource,
    /// HTMLFile 传输。
    HtmlFile,
}

impl TransportType {
    /// 返回 Spring 协议字符串值。
    #[must_use]
    pub const fn value(self) -> &'static str {
        match self {
            Self::WebSocket => "websocket",
            Self::Xhr => "xhr",
            Self::XhrSend => "xhr_send",
            Self::XhrStreaming => "xhr_streaming",
            Self::EventSource => "eventsource",
            Self::HtmlFile => "htmlfile",
        }
    }

    /// 返回 HTTP 方法。
    #[must_use]
    pub const fn http_method(self) -> Method {
        match self {
            Self::WebSocket | Self::EventSource | Self::HtmlFile => Method::GET,
            Self::Xhr | Self::XhrSend | Self::XhrStreaming => Method::POST,
        }
    }

    /// 返回 Spring header hints（cors/jsessionid/no_cache/origin）。
    #[must_use]
    const fn header_hints(self) -> &'static [&'static str] {
        match self {
            Self::WebSocket => &["origin"],
            Self::Xhr | Self::XhrSend | Self::XhrStreaming => &["cors", "jsessionid", "no_cache"],
            Self::EventSource => &["origin", "jsessionid", "no_cache"],
            Self::HtmlFile => &["cors", "jsessionid", "no_cache"],
        }
    }

    /// 是否发送 no-cache 头。
    #[must_use]
    pub fn sends_no_cache_instruction(self) -> bool {
        self.header_hints().contains(&"no_cache")
    }

    /// 是否发送 session cookie。
    #[must_use]
    pub fn sends_session_cookie(self) -> bool {
        self.header_hints().contains(&"jsessionid")
    }

    /// 是否支持 CORS。
    #[must_use]
    pub fn supports_cors(self) -> bool {
        self.header_hints().contains(&"cors")
    }

    /// 是否支持 Origin 检查。
    #[must_use]
    pub fn supports_origin(self) -> bool {
        let hints = self.header_hints();
        hints.contains(&"cors") || hints.contains(&"origin")
    }

    /// Spring `TransportType.fromValue(String)`：按 value 字符串查找。
    #[must_use]
    pub fn from_value(value: &str) -> Option<Self> {
        static MAP: OnceLock<HashMap<&'static str, TransportType>> = OnceLock::new();
        let map = MAP.get_or_init(|| {
            let mut map = HashMap::new();
            for variant in [
                Self::WebSocket,
                Self::Xhr,
                Self::XhrSend,
                Self::XhrStreaming,
                Self::EventSource,
                Self::HtmlFile,
            ] {
                map.insert(variant.value(), variant);
            }
            map
        });
        map.get(value).copied()
    }
}

impl std::fmt::Display for TransportType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_resolves_known_types() {
        assert_eq!(
            TransportType::from_value("websocket"),
            Some(TransportType::WebSocket)
        );
        assert_eq!(
            TransportType::from_value("xhr_streaming"),
            Some(TransportType::XhrStreaming)
        );
        assert_eq!(TransportType::from_value("unknown"), None);
    }

    #[test]
    fn header_hints_match_spring_definitions() {
        assert!(TransportType::XhrStreaming.supports_cors());
        assert!(TransportType::XhrStreaming.sends_session_cookie());
        assert!(TransportType::XhrStreaming.sends_no_cache_instruction());
        assert!(TransportType::WebSocket.supports_origin());
        assert!(!TransportType::WebSocket.supports_cors());
    }
}
