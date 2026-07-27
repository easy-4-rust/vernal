//! 对应 Java 类：org.springframework.web.socket.sockjs.SockJsService
//!
//! SockJS HTTP 请求处理入口。Spring 与 Servlet 强耦合（ServerHttpRequest/Response），
//! Vernal 通过 async trait + 框架无关 `SockJsRequest`/`SockJsResponse` 解耦。

use std::{future::Future, pin::Pin, sync::Arc};

use crate::WebSocketHandler;
use crate::sockjs::sockjs_error::SockJsError;

/// SockJS HTTP 请求视图。
pub struct SockJsRequest {
    /// SockJS path（去掉服务前缀后的剩余路径）。
    pub sockjs_path: String,
    /// HTTP 方法。
    pub method: http::Method,
    /// 请求头。
    pub headers: http::HeaderMap,
}

/// SockJS HTTP 响应 writer SPI。
pub trait SockJsResponseWriter: Send + Sync {
    /// 设置 HTTP 状态码。
    fn set_status(&self, status: u16);
    /// 设置 header。
    fn set_header(&self, name: &str, value: &str);
    /// 写入 body 并结束响应。
    fn write_body<'a>(
        &'a self,
        body: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), SockJsError>> + Send + 'a>>;
}

/// SockJS 服务 future。
pub type SockJsServiceFuture = Pin<Box<dyn Future<Output = Result<(), SockJsError>> + Send>>;

/// SockJS 服务 SPI。
pub trait SockJsService: Send + Sync {
    /// 处理 SockJS HTTP 请求。
    fn handle_request(
        &self,
        request: SockJsRequest,
        response: Arc<dyn SockJsResponseWriter>,
        handler: Arc<dyn WebSocketHandler>,
    ) -> SockJsServiceFuture;
}
