//! 对应 Java 类：org.springframework.web.socket.sockjs.client.SockJsClient
//!
//! SockJS 客户端主入口：按 transport 优先级尝试连接，失败则降级到下一个 transport。
//! 对标 Spring `SockJsClient`。

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::WebSocketHandler;
use crate::sockjs::client::abstract_client_sockjs_session::ClientSockJsSession;
use crate::sockjs::client::default_transport_request::DefaultTransportRequest;
use crate::sockjs::client::info_receiver::InfoReceiver;
use crate::sockjs::client::sockjs_url_info::SockJsUrlInfo;
use crate::sockjs::client::transport::Transport;
use crate::sockjs::frame::json_sockjs_message_codec::JsonSockJsMessageCodec;
use crate::sockjs::frame::sockjs_message_codec::SockJsMessageCodec;
use crate::sockjs::transport::transport_type::TransportType;

/// SockJS 客户端。
pub struct SockJsClient {
    transports: Vec<Arc<dyn Transport>>,
    info_receiver: Arc<dyn InfoReceiver>,
    codec: Arc<dyn SockJsMessageCodec>,
    /// 已建立的 session。
    session: Mutex<Option<Arc<ClientSockJsSession>>>,
}

impl SockJsClient {
    /// 创建客户端。
    #[must_use]
    pub fn new(transports: Vec<Arc<dyn Transport>>, info_receiver: Arc<dyn InfoReceiver>) -> Self {
        Self {
            transports,
            info_receiver,
            codec: Arc::new(JsonSockJsMessageCodec::new()),
            session: Mutex::new(None),
        }
    }

    /// 返回已注册的 transport 数。
    #[must_use]
    pub fn transport_count(&self) -> usize {
        self.transports.len()
    }

    /// 返回已注册 transport 支持的全部 transport 类型。
    pub fn supported_transport_types(&self) -> Vec<TransportType> {
        self.transports
            .iter()
            .flat_map(|t| t.transport_types())
            .collect()
    }

    /// 连接到 SockJS 服务端。
    ///
    /// 对标 Spring `SockJsClient`：
    /// 1. 获取 `/info`，检查 `websocket` 能力；
    /// 2. 按 transport 优先级逐个构造 `DefaultTransportRequest` 并调用 `connect_async`；
    /// 3. 成功则建立 session；失败则降级到下一个 transport；
    /// 4. 全部失败则返回错误。
    ///
    /// # Errors
    ///
    /// 当所有 transport 都失败时返回错误。
    pub async fn connect(
        &self,
        url: &str,
        handler: Arc<dyn WebSocketHandler>,
    ) -> Result<Arc<ClientSockJsSession>, String> {
        let url_info = SockJsUrlInfo::new(url);
        let session_id = url_info.session_id();
        let session = Arc::new(ClientSockJsSession::new(
            session_id.clone(),
            Arc::clone(&self.codec),
            Arc::clone(&handler),
        ));

        // 1. 获取 /info
        let info = self.info_receiver.fetch_info(&url_info.info_url()).await?;

        // 2. 按 transport 优先级尝试连接（对标 Spring fallback chain）
        let mut last_error = String::new();
        for transport in &self.transports {
            let transport_types = transport.transport_types();

            // 如果服务端不支持 WebSocket 且该 transport 只支持 WebSocket，跳过
            if !info.websocket
                && transport_types
                    .iter()
                    .all(|t| *t == TransportType::WebSocket)
            {
                continue;
            }

            // 为该 transport 的每个候选类型构造 TransportRequest
            for transport_type in &transport_types {
                let request = DefaultTransportRequest::new(
                    SockJsUrlInfo::with_ids(url, url_info.server_id(), url_info.session_id()),
                    *transport_type,
                    Arc::clone(&self.codec),
                );

                // 尝试连接
                match transport
                    .connect_async(&request, Arc::clone(&handler))
                    .await
                {
                    Ok(()) => {
                        // 连接成功：建立 session
                        session.open().await;
                        *self.session.lock().await = Some(Arc::clone(&session));
                        return Ok(session);
                    }
                    Err(error) => {
                        last_error = format!("Transport {:?} failed: {error}", transport_type);
                        // 降级到下一个 transport type
                    }
                }
            }
        }

        Err(format!("All transports failed. Last error: {last_error}"))
    }

    /// 返回 session（如果已连接）。
    pub async fn session(&self) -> Option<Arc<ClientSockJsSession>> {
        self.session.lock().await.clone()
    }
}

impl std::fmt::Debug for SockJsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SockJsClient")
            .field("transport_count", &self.transports.len())
            .finish_non_exhaustive()
    }
}
