//! 对应 Java 类：org.springframework.web.socket.server.support.HttpSessionHandshakeInterceptor
//!
//! 把 HTTP 会话属性复制到握手 attributes。Spring 依赖 `jakarta.servlet.HttpSession`，
//! Vernal 使用框架无关的 `HttpSessionAttributes` 抽象表达等价语义，便于从任何
//! HTTP 框架（axum/hyper/actix）桥接。

use std::collections::BTreeMap;

use crate::server::{HandshakeContext, HandshakeInterceptor};

/// HTTP session 属性的框架无关映射。
pub type HttpSessionAttributes = BTreeMap<String, String>;

/// HTTP session ID 在握手 attributes 中的 key。
pub const HTTP_SESSION_ID_ATTR_NAME: &str = "HTTP.SESSION.ID";

/// HTTP session 视图：id + 可选属性。
#[derive(Debug, Clone, Default)]
pub struct HttpSession {
    /// session id（可能为 None 表示无 session）。
    pub id: Option<String>,
    /// session 属性。
    pub attributes: HttpSessionAttributes,
}

impl HttpSession {
    /// 创建空 session。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// 把 session 属性复制到握手 attributes。
#[derive(Clone)]
pub struct HttpSessionHandshakeInterceptor {
    attribute_names: Vec<String>,
    copy_all_attributes: bool,
    copy_http_session_id: bool,
    http_session: HttpSession,
}

impl HttpSessionHandshakeInterceptor {
    /// 默认复制所有属性 + session id。
    #[must_use]
    pub fn new() -> Self {
        Self {
            attribute_names: Vec::new(),
            copy_all_attributes: true,
            copy_http_session_id: true,
            http_session: HttpSession::new(),
        }
    }

    /// 仅复制指定属性。
    #[must_use]
    pub fn with_attribute_names(mut self, names: impl IntoIterator<Item = String>) -> Self {
        self.attribute_names = names.into_iter().collect();
        self.copy_all_attributes = false;
        self
    }

    /// 是否复制全部属性。
    #[must_use]
    pub const fn copy_all_attributes(&self) -> bool {
        self.copy_all_attributes
    }

    /// 设置是否复制全部属性。
    #[must_use]
    pub const fn set_copy_all_attributes(mut self, copy: bool) -> Self {
        self.copy_all_attributes = copy;
        self
    }

    /// 是否复制 session id。
    #[must_use]
    pub const fn copy_http_session_id(&self) -> bool {
        self.copy_http_session_id
    }

    /// 设置是否复制 session id。
    #[must_use]
    pub const fn set_copy_http_session_id(mut self, copy: bool) -> Self {
        self.copy_http_session_id = copy;
        self
    }

    /// 绑定 HTTP session 视图。
    #[must_use]
    pub fn with_http_session(mut self, session: HttpSession) -> Self {
        self.http_session = session;
        self
    }
}

impl Default for HttpSessionHandshakeInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl HandshakeInterceptor for HttpSessionHandshakeInterceptor {
    fn before_handshake<'a>(
        &'a self,
        context: &'a mut HandshakeContext,
        _handler: &'a std::sync::Arc<dyn crate::WebSocketHandler>,
    ) -> crate::server::BeforeHandshakeFuture<'a> {
        Box::pin(async move {
            if self.copy_all_attributes {
                for (key, value) in &self.http_session.attributes {
                    context.attributes.insert(key.clone(), value.clone());
                }
            } else {
                for name in &self.attribute_names {
                    if let Some(value) = self.http_session.attributes.get(name) {
                        context.attributes.insert(name.clone(), value.clone());
                    }
                }
            }
            if self.copy_http_session_id
                && let Some(id) = &self.http_session.id
            {
                context
                    .attributes
                    .insert(HTTP_SESSION_ID_ATTR_NAME.to_owned(), id.clone());
            }
            Ok(true)
        })
    }
}
