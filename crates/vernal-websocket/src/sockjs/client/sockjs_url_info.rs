//! 对应 Java 类：org.springframework.web.socket.sockjs.client.SockJsUrlInfo
//!
//! SockJS URL 信息容器：从基础 URL 派生 info URL 和 transport URL。

use std::sync::Mutex;

use crate::sockjs::transport::transport_type::TransportType;

/// SockJS URL 信息。
pub struct SockJsUrlInfo {
    sockjs_url: String,
    server_id: Mutex<Option<String>>,
    session_id: Mutex<Option<String>>,
}

impl SockJsUrlInfo {
    /// 创建 URL 信息。
    #[must_use]
    pub fn new(sockjs_url: impl Into<String>) -> Self {
        Self {
            sockjs_url: sockjs_url.into(),
            server_id: Mutex::new(None),
            session_id: Mutex::new(None),
        }
    }

    /// 使用指定 server id 和 session id 创建。
    #[must_use]
    pub fn with_ids(
        sockjs_url: impl Into<String>,
        server_id: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            sockjs_url: sockjs_url.into(),
            server_id: Mutex::new(Some(server_id.into())),
            session_id: Mutex::new(Some(session_id.into())),
        }
    }

    /// 返回基础 SockJS URL。
    #[must_use]
    pub fn sockjs_url(&self) -> &str {
        &self.sockjs_url
    }

    /// 返回 server id（对标 Spring `getServerId`：随机 0-999）。
    pub fn server_id(&self) -> String {
        let mut guard = self.server_id.lock().expect("server_id poisoned");
        if guard.is_none() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos();
            *guard = Some(format!("{}", now % 1000));
        }
        guard.clone().unwrap_or_default()
    }

    /// 返回 session id（对标 Spring `getSessionId`：UUID 去掉 `-`）。
    pub fn session_id(&self) -> String {
        let mut guard = self.session_id.lock().expect("session_id poisoned");
        if guard.is_none() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            *guard = Some(format!("{:x}{:x}", now.as_nanos(), now.subsec_nanos()));
        }
        guard.clone().unwrap_or_default()
    }

    /// 返回 info URL：`{baseUrl}/info`。
    #[must_use]
    pub fn info_url(&self) -> String {
        format!("{}/info", self.sockjs_url.trim_end_matches('/'))
    }

    /// 返回 transport URL：`{baseUrl}/{serverId}/{sessionId}/{transport}`。
    #[must_use]
    pub fn transport_url(&self, transport_type: TransportType) -> String {
        format!(
            "{}/{}/{}/{}",
            self.sockjs_url.trim_end_matches('/'),
            self.server_id(),
            self.session_id(),
            transport_type.value()
        )
    }
}

impl std::fmt::Debug for SockJsUrlInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SockJsUrlInfo")
            .field("sockjs_url", &self.sockjs_url)
            .finish_non_exhaustive()
    }
}
