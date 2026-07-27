//! WebSocket 握手校验。

use http::{HeaderMap, HeaderValue, Method, StatusCode, Uri};

use crate::{WebSocketError, negotiate_subprotocol};

/// WebSocket 握手请求快照。
#[derive(Debug, Clone)]
pub struct HandshakeRequest {
    /// 请求方法。
    pub method: Method,
    /// 请求 URI。
    pub uri: Uri,
    /// 请求头。
    pub headers: HeaderMap,
}

impl HandshakeRequest {
    /// 从 HTTP 元数据创建快照。
    #[must_use]
    pub fn new(method: Method, uri: Uri, headers: HeaderMap) -> Self {
        Self {
            method,
            uri,
            headers,
        }
    }

    /// 验证 RFC 6455 基本握手头。
    pub fn validate(&self) -> Result<(), WebSocketError> {
        if self.method != Method::GET {
            return Err(WebSocketError::Handshake {
                status: StatusCode::METHOD_NOT_ALLOWED.as_u16(),
                reason: "握手必须使用 GET".into(),
            });
        }
        let upgrade = header_str(&self.headers, "upgrade");
        if !upgrade.eq_ignore_ascii_case("websocket") {
            return Err(WebSocketError::Handshake {
                status: StatusCode::BAD_REQUEST.as_u16(),
                reason: "缺少 Upgrade: websocket".into(),
            });
        }
        if !header_contains_token(&self.headers, "connection", "upgrade") {
            return Err(WebSocketError::Handshake {
                status: StatusCode::BAD_REQUEST.as_u16(),
                reason: "缺少 Connection: Upgrade".into(),
            });
        }
        if header_str(&self.headers, "sec-websocket-version") != "13" {
            return Err(WebSocketError::Handshake {
                status: StatusCode::UPGRADE_REQUIRED.as_u16(),
                reason: "只支持 WebSocket version 13".into(),
            });
        }
        if self.headers.get("sec-websocket-key").is_none() {
            return Err(WebSocketError::Handshake {
                status: StatusCode::BAD_REQUEST.as_u16(),
                reason: "缺少 Sec-WebSocket-Key".into(),
            });
        }
        Ok(())
    }

    /// 按客户端请求头协商子协议。
    #[must_use]
    pub fn negotiate_subprotocol(&self, supported: &[String]) -> Option<String> {
        let requested = self.headers.get("sec-websocket-protocol")?.to_str().ok()?;
        let requested = requested.split(',').map(str::trim).collect::<Vec<_>>();
        negotiate_subprotocol(&requested, supported)
    }
}

/// Origin 白名单策略。
#[derive(Debug, Clone, Default)]
pub struct OriginPolicy {
    allowed: Vec<String>,
    allow_missing: bool,
}

impl OriginPolicy {
    /// 创建拒绝所有 Origin 的策略。
    #[must_use]
    pub fn deny_all() -> Self {
        Self::default()
    }

    /// 创建允许任意 Origin 的策略。
    #[must_use]
    pub fn allow_all() -> Self {
        Self {
            allowed: vec!["*".into()],
            allow_missing: true,
        }
    }

    /// 添加允许的 Origin。
    #[must_use]
    pub fn allow(mut self, origin: impl Into<String>) -> Self {
        self.allowed.push(origin.into());
        self
    }

    /// 设置是否允许缺失 Origin。
    #[must_use]
    pub fn allow_missing(mut self, allow: bool) -> Self {
        self.allow_missing = allow;
        self
    }

    /// 校验 Origin。
    pub fn validate(&self, headers: &HeaderMap) -> Result<(), WebSocketError> {
        let Some(origin) = headers.get("origin").and_then(|value| value.to_str().ok()) else {
            return if self.allow_missing {
                Ok(())
            } else {
                Err(WebSocketError::Handshake {
                    status: 403,
                    reason: "Origin header is required".into(),
                })
            };
        };
        if self
            .allowed
            .iter()
            .any(|allowed| allowed == "*" || allowed == origin)
        {
            Ok(())
        } else {
            Err(WebSocketError::Handshake {
                status: 403,
                reason: format!("Origin 不被允许: {origin}"),
            })
        }
    }
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> &'a str {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

fn header_contains_token(headers: &HeaderMap, name: &str, token: &str) -> bool {
    header_str(headers, name)
        .split(',')
        .any(|value| value.trim().eq_ignore_ascii_case(token))
}

/// 返回一个 ASCII header 值，便于测试构造请求。
#[must_use]
pub fn header_value(value: &str) -> HeaderValue {
    HeaderValue::from_str(value).unwrap_or_else(|_| HeaderValue::from_static(""))
}
