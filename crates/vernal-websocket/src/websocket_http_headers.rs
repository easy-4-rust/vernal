//! 对应 Java 类：org.springframework.web.socket.WebSocketHttpHeaders
//!
//! 仅持有 WebSocket 握手相关 header 的 typed wrapper。底层 `HeaderMap`
//! 来自 `http` crate；提供 `Sec-WebSocket-Accept/Extensions/Key/Protocol/Version`
//! 的访问与设置。
//!
//! 对标 Spring `WebSocketHttpHeaders`，但底层使用 `http::HeaderMap` 而非
//! Spring `HttpHeaders`。

use http::HeaderMap;
use http::header::HeaderName;

use crate::{WebSocketError, websocket_extension::WebSocketExtension};

/// `Sec-WebSocket-Accept` header name.
pub const SEC_WEBSOCKET_ACCEPT: &str = "sec-websocket-accept";
/// `Sec-WebSocket-Extensions` header name.
pub const SEC_WEBSOCKET_EXTENSIONS: &str = "sec-websocket-extensions";
/// `Sec-WebSocket-Key` header name.
pub const SEC_WEBSOCKET_KEY: &str = "sec-websocket-key";
/// `Sec-WebSocket-Protocol` header name.
pub const SEC_WEBSOCKET_PROTOCOL: &str = "sec-websocket-protocol";
/// `Sec-WebSocket-Version` header name.
pub const SEC_WEBSOCKET_VERSION: &str = "sec-websocket-version";

/// WebSocket 握手 header wrapper。
#[derive(Debug, Clone, Default)]
pub struct WebSocketHttpHeaders {
    headers: HeaderMap,
}

impl WebSocketHttpHeaders {
    /// 创建空 header 集合。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 包装现有 `HeaderMap`。
    #[must_use]
    pub fn from_headers(headers: HeaderMap) -> Self {
        Self { headers }
    }

    /// 返回底层 `HeaderMap` 引用。
    #[must_use]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// 返回可变 `HeaderMap` 引用。
    #[must_use]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// 返回底层 `HeaderMap`（消耗 wrapper）。
    #[must_use]
    pub fn into_headers(self) -> HeaderMap {
        self.headers
    }

    /// 读取 `Sec-WebSocket-Accept`。
    #[must_use]
    pub fn sec_websocket_accept(&self) -> Option<String> {
        self.first_string(SEC_WEBSOCKET_ACCEPT)
    }

    /// 设置 `Sec-WebSocket-Accept`。
    pub fn set_sec_websocket_accept(&mut self, value: Option<&str>) {
        self.set_first(SEC_WEBSOCKET_ACCEPT, value);
    }

    /// 读取 `Sec-WebSocket-Extensions`，按 Spring 行为解析为 `WebSocketExtension` 列表。
    ///
    /// # Errors
    ///
    /// 当 header 格式不合法时返回错误。
    pub fn sec_websocket_extensions(&self) -> Result<Vec<WebSocketExtension>, WebSocketError> {
        let mut result = Vec::new();
        for value in &self
            .headers
            .get_all(HeaderName::from_static(SEC_WEBSOCKET_EXTENSIONS))
        {
            let text = value.to_str().map_err(|_| {
                WebSocketError::protocol(
                    crate::CloseCode::ProtocolError,
                    "Sec-WebSocket-Extensions 不是有效 ASCII",
                )
            })?;
            result.extend(WebSocketExtension::parse_extensions(text)?);
        }
        Ok(result)
    }

    /// 设置 `Sec-WebSocket-Extensions`。
    ///
    /// # Errors
    ///
    /// 当扩展名或参数无法编码为合法 header 值时返回错误。
    pub fn set_sec_websocket_extensions(
        &mut self,
        extensions: &[WebSocketExtension],
    ) -> Result<(), WebSocketError> {
        self.headers
            .remove(HeaderName::from_static(SEC_WEBSOCKET_EXTENSIONS));
        let joined = extensions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        if joined.is_empty() {
            return Ok(());
        }
        let value = http::HeaderValue::from_str(&joined).map_err(|_| {
            WebSocketError::protocol(
                crate::CloseCode::ProtocolError,
                "Sec-WebSocket-Extensions 不是有效 header 值",
            )
        })?;
        self.headers
            .insert(HeaderName::from_static(SEC_WEBSOCKET_EXTENSIONS), value);
        Ok(())
    }

    /// 读取 `Sec-WebSocket-Key`。
    #[must_use]
    pub fn sec_websocket_key(&self) -> Option<String> {
        self.first_string(SEC_WEBSOCKET_KEY)
    }

    /// 设置 `Sec-WebSocket-Key`。
    pub fn set_sec_websocket_key(&mut self, value: Option<&str>) {
        self.set_first(SEC_WEBSOCKET_KEY, value);
    }

    /// 读取 `Sec-WebSocket-Protocol`，按 Spring `getSecWebSocketProtocol` 行为拆分。
    #[must_use]
    pub fn sec_websocket_protocol(&self) -> Vec<String> {
        let values = self
            .headers
            .get_all(HeaderName::from_static(SEC_WEBSOCKET_PROTOCOL))
            .iter()
            .filter_map(|value| value.to_str().ok().map(str::to_owned))
            .collect::<Vec<_>>();
        if values.is_empty() {
            return Vec::new();
        }
        if values.len() == 1 {
            values[0]
                .split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(str::to_owned)
                .collect()
        } else {
            values
        }
    }

    /// 设置 `Sec-WebSocket-Protocol`（逗号拼接）。
    pub fn set_sec_websocket_protocol(&mut self, protocols: &[&str]) {
        self.headers
            .remove(HeaderName::from_static(SEC_WEBSOCKET_PROTOCOL));
        let joined = protocols.join(", ");
        if joined.is_empty() {
            return;
        }
        if let Ok(value) = http::HeaderValue::from_str(&joined) {
            self.headers
                .insert(HeaderName::from_static(SEC_WEBSOCKET_PROTOCOL), value);
        }
    }

    /// 读取 `Sec-WebSocket-Version`。
    #[must_use]
    pub fn sec_websocket_version(&self) -> Option<String> {
        self.first_string(SEC_WEBSOCKET_VERSION)
    }

    /// 设置 `Sec-WebSocket-Version`。
    pub fn set_sec_websocket_version(&mut self, value: Option<&str>) {
        self.set_first(SEC_WEBSOCKET_VERSION, value);
    }

    fn first_string(&self, name: &str) -> Option<String> {
        self.headers
            .get(HeaderName::try_from(name).ok()?)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    }

    fn set_first(&mut self, name: &str, value: Option<&str>) {
        let Ok(header_name) = HeaderName::try_from(name) else {
            return;
        };
        self.headers.remove(&header_name);
        if let Some(value) = value
            && let Ok(header_value) = http::HeaderValue::from_str(value)
        {
            self.headers.insert(header_name, header_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn sec_websocket_protocol_splits_single_header_into_list() {
        let mut headers = WebSocketHttpHeaders::new();
        headers.set_sec_websocket_protocol(&["chat", "v12.stomp"]);
        assert_eq!(
            headers.sec_websocket_protocol(),
            vec!["chat".to_string(), "v12.stomp".to_string()]
        );
    }

    #[test]
    fn sec_websocket_extensions_round_trip() {
        let mut headers = WebSocketHttpHeaders::new();
        let ext = WebSocketExtension::new(
            "permessage-deflate",
            Some(BTreeMap::from([(
                "server_max_window_bits".to_string(),
                "10".to_string(),
            )])),
        )
        .unwrap();
        headers
            .set_sec_websocket_extensions(&[ext.clone()])
            .unwrap();
        let parsed = headers.sec_websocket_extensions().unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], ext);
    }

    #[test]
    fn accept_key_get_and_set() {
        let mut headers = WebSocketHttpHeaders::new();
        headers.set_sec_websocket_accept(Some("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
        assert_eq!(
            headers.sec_websocket_accept().as_deref(),
            Some("s3pPLMBiTxaQ9kYGzzhZRbK+xOo=")
        );
        headers.set_sec_websocket_accept(None);
        assert!(headers.sec_websocket_accept().is_none());
    }
}
