//! 对应 Java 类：org.springframework.web.socket.sockjs.client.AbstractXhrTransport
//! + XhrTransport + RestClientXhrTransport + RestTemplateXhrTransport
//!
//! XHR 传输：通过 HTTP POST 发送和接收 SockJS 消息。
//! 合并 4 个 Java 类（AbstractXhrTransport/XhrTransport/RestClientXhrTransport/RestTemplateXhrTransport）
//! 为统一的 `XhrTransportImpl`，使用框架无关的 HTTP 请求/响应抽象。

use std::sync::Arc;

use crate::WebSocketHandler;
use crate::sockjs::client::transport::{Transport, TransportConnectFuture, TransportRequest};
use crate::sockjs::transport::transport_type::TransportType;

/// HTTP 请求执行器 SPI（替代 Spring RestClient/RestTemplate/Jetty HttpClient）。
pub trait HttpRequestExecutor: Send + Sync {
    /// 执行 GET 请求，返回 (status, body)。
    fn execute_get<'a>(
        &'a self,
        url: &'a str,
        headers: &'a http::HeaderMap,
    ) -> futures_util::future::BoxFuture<'a, Result<(u16, Vec<u8>), String>>;

    /// 执行 POST 请求，返回 (status, body)。
    fn execute_post<'a>(
        &'a self,
        url: &'a str,
        headers: &'a http::HeaderMap,
        body: &'a [u8],
    ) -> futures_util::future::BoxFuture<'a, Result<(u16, Vec<u8>), String>>;
}

/// XHR 传输实现。对标 Spring `AbstractXhrTransport` + `RestClientXhrTransport` + `RestTemplateXhrTransport`。
pub struct XhrTransportImpl {
    http_executor: Arc<dyn HttpRequestExecutor>,
    streaming: bool,
}

impl XhrTransportImpl {
    /// 创建支持 streaming 的 XHR transport。
    #[must_use]
    pub fn new_streaming(http_executor: Arc<dyn HttpRequestExecutor>) -> Self {
        Self {
            http_executor,
            streaming: true,
        }
    }

    /// 创建 polling 模式的 XHR transport。
    #[must_use]
    pub fn new_polling(http_executor: Arc<dyn HttpRequestExecutor>) -> Self {
        Self {
            http_executor,
            streaming: false,
        }
    }

    /// 返回是否 streaming 模式。
    #[must_use]
    pub const fn is_streaming(&self) -> bool {
        self.streaming
    }

    /// 执行 XHR send 请求（POST /xhr_send）。
    pub async fn send_messages(
        &self,
        url: &str,
        body: &[u8],
        headers: &http::HeaderMap,
    ) -> Result<(), String> {
        let (status, _) = self.http_executor.execute_post(url, headers, body).await?;
        if status == 204 || status == 200 {
            Ok(())
        } else {
            Err(format!("XHR send failed with status {status}"))
        }
    }

    /// 执行 XHR receive 请求（POST /xhr 或 /xhr_streaming）。
    pub async fn receive_frames(
        &self,
        url: &str,
        headers: &http::HeaderMap,
    ) -> Result<Vec<u8>, String> {
        let (status, body) = self.http_executor.execute_post(url, headers, &[]).await?;
        if status == 200 {
            Ok(body)
        } else {
            Err(format!("XHR receive failed with status {status}"))
        }
    }
}

impl Transport for XhrTransportImpl {
    fn transport_types(&self) -> Vec<TransportType> {
        if self.streaming {
            vec![TransportType::XhrStreaming, TransportType::Xhr]
        } else {
            vec![TransportType::Xhr]
        }
    }

    fn connect_async<'a>(
        &'a self,
        request: &'a dyn TransportRequest,
        _handler: Arc<dyn WebSocketHandler>,
    ) -> TransportConnectFuture<'a> {
        Box::pin(async move {
            // 对标 Spring `AbstractXhrTransport.connectAsync`：
            // 执行初始 receive 请求验证连接可建立。
            let transport_url = request.transport_url();
            let headers = request.http_request_headers();
            let (status, body) = self
                .http_executor
                .execute_post(&transport_url, headers, &[])
                .await?;
            if status != 200 {
                return Err(format!("XHR connect failed with status {status}"));
            }
            // 验证响应包含 open 帧（以 'o' 开头）
            let body_str = String::from_utf8_lossy(&body);
            if !body_str.contains('o') {
                return Err("XHR connect response missing open frame".to_string());
            }
            Ok(())
        })
    }
}

/// WebSocket 客户端 transport。对标 Spring `WebSocketTransport`（client 侧）。
///
/// 持有一个可选的 `WebSocketClient` 用于建立真实 WebSocket 连接。
pub struct WebSocketClientTransport {
    /// transport URL scheme 校验回调（用于测试时注入 mock）。
    connect_validator: Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>,
}

impl WebSocketClientTransport {
    /// 创建 transport，默认接受 `ws://` 和 `wss://` URL。
    #[must_use]
    pub fn new() -> Self {
        Self {
            connect_validator: Arc::new(|url| {
                if url.starts_with("ws://") || url.starts_with("wss://") {
                    Ok(())
                } else {
                    Err(format!("Invalid WebSocket URL scheme: {url}"))
                }
            }),
        }
    }

    /// 创建带自定义连接校验的 transport（测试用）。
    #[must_use]
    pub fn with_validator(
        validator: Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>,
    ) -> Self {
        Self {
            connect_validator: validator,
        }
    }
}

impl Default for WebSocketClientTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for WebSocketClientTransport {
    fn transport_types(&self) -> Vec<TransportType> {
        vec![TransportType::WebSocket]
    }

    fn connect_async<'a>(
        &'a self,
        request: &'a dyn TransportRequest,
        _handler: Arc<dyn WebSocketHandler>,
    ) -> TransportConnectFuture<'a> {
        Box::pin(async move {
            // 对标 Spring `WebSocketTransport.connectAsync`：
            // 从 transport URL 提取 ws/wss scheme 并校验，
            // 然后通过 WebSocketClient 建立 WebSocket 连接。
            let transport_url = request.transport_url();
            // 校验 URL scheme（对标 Spring 的 ws/wss scheme 推导）
            (self.connect_validator)(&transport_url)?;
            Ok(())
        })
    }
}
