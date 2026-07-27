//! 对应 Java 类：org.springframework.web.socket.WebSocketExtension
//!
//! 表示一个 RFC 6455 WebSocket 扩展。扩展在握手阶段协商，可携带参数。
//! 对标 Spring `WebSocketExtension`，包括 `parseExtensions` 解析 header 行为。

use std::collections::BTreeMap;
use std::fmt;

use crate::WebSocketError;

/// WebSocket 扩展。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketExtension {
    /// 扩展名（非空）。
    name: String,
    /// 参数映射（大小写不敏感、有序）。
    parameters: BTreeMap<String, String>,
}

impl WebSocketExtension {
    /// 创建扩展。
    ///
    /// # Errors
    ///
    /// 当扩展名为空时返回错误。
    pub fn new(
        name: impl Into<String>,
        parameters: Option<BTreeMap<String, String>>,
    ) -> Result<Self, WebSocketError> {
        let name = name.into();
        if name.is_empty() {
            return Err(WebSocketError::protocol(
                crate::CloseCode::ProtocolError,
                "Extension name must not be empty",
            ));
        }
        Ok(Self {
            name,
            parameters: parameters.unwrap_or_default(),
        })
    }

    /// 返回扩展名。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回参数。
    #[must_use]
    pub fn parameters(&self) -> &BTreeMap<String, String> {
        &self.parameters
    }

    /// 解析 `Sec-WebSocket-Extensions` header 值。
    ///
    /// # Errors
    ///
    /// 当格式不合法时返回错误。
    pub fn parse_extensions(header: &str) -> Result<Vec<Self>, WebSocketError> {
        let trimmed = header.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        trimmed.split(',').map(Self::parse_extension).collect()
    }

    fn parse_extension(token: &str) -> Result<Self, WebSocketError> {
        if token.contains(',') {
            return Err(WebSocketError::protocol(
                crate::CloseCode::ProtocolError,
                format!("Expected single extension value: [{token}]"),
            ));
        }
        let mut parts = token
            .split(';')
            .map(str::trim)
            .filter(|part| !part.is_empty());
        let name = parts
            .next()
            .ok_or_else(|| {
                WebSocketError::protocol(
                    crate::CloseCode::ProtocolError,
                    format!("Extension value is empty: [{token}]"),
                )
            })?
            .to_owned();
        let mut parameters = BTreeMap::new();
        for parameter in parts {
            match parameter.find('=') {
                Some(eq_index) => {
                    let attribute = parameter[..eq_index].to_owned();
                    let value = parameter[eq_index + 1..].to_owned();
                    parameters.insert(attribute, value);
                }
                None => {
                    // RFC 7230 允许没有值的参数；Spring 仅以 attribute 为 key、空字符串为 value。
                    parameters.insert(parameter.to_owned(), String::new());
                }
            }
        }
        Self::new(name, Some(parameters))
    }
}

impl fmt::Display for WebSocketExtension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)?;
        for (key, value) in &self.parameters {
            write!(formatter, ";{key}={value}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_extension_with_parameters() {
        let extensions = WebSocketExtension::parse_extensions(
            "permessage-deflate; client_no_context_takeover; server_max_window_bits=10",
        )
        .unwrap();
        assert_eq!(extensions.len(), 1);
        let ext = &extensions[0];
        assert_eq!(ext.name(), "permessage-deflate");
        assert_eq!(
            ext.parameters()
                .get("client_no_context_takeover")
                .map(String::as_str),
            Some("")
        );
        assert_eq!(
            ext.parameters()
                .get("server_max_window_bits")
                .map(String::as_str),
            Some("10")
        );
    }

    #[test]
    fn parse_multiple_extensions_preserves_order() {
        let extensions = WebSocketExtension::parse_extensions("foo, bar; baz=qux").unwrap();
        assert_eq!(extensions.len(), 2);
        assert_eq!(extensions[0].name(), "foo");
        assert_eq!(extensions[1].name(), "bar");
        assert_eq!(
            extensions[1].parameters().get("baz").map(String::as_str),
            Some("qux")
        );
    }

    #[test]
    fn parse_empty_header_returns_empty() {
        assert!(WebSocketExtension::parse_extensions("").unwrap().is_empty());
        assert!(
            WebSocketExtension::parse_extensions("   ")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn reject_empty_extension_name() {
        assert!(WebSocketExtension::new("", None).is_err());
    }

    #[test]
    fn display_round_trips_extension_format() {
        let ext = WebSocketExtension::new(
            "permessage-deflate",
            Some(BTreeMap::from([(
                "server_max_window_bits".to_string(),
                "10".to_string(),
            )])),
        )
        .unwrap();
        assert_eq!(
            ext.to_string(),
            "permessage-deflate;server_max_window_bits=10"
        );
    }
}
