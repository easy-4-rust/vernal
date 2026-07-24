//! Hyper 与 Vernal HTTP 合同之间的桥接对象。

use futures_util::StreamExt;
use http::{Request, Response};
use http_body_util::BodyStream;
use hyper::body::Incoming;
use tokio_util::sync::CancellationToken;
use vernal_http::{HttpBody, HttpBodyError, HttpRequest, HttpResponse};

/// 在 Hyper 原生请求和 Vernal 协议对象之间执行无损转换。
///
/// 该对象没有运行时状态，也不缓冲 Body。Hyper 的数据帧和 Trailer 通过
/// `BodyStream` 原样转发；Method、URI、Version、Header 与 Extensions 则继续
/// 使用 `http` crate 的标准 Parts。
pub struct HyperBridge;

impl HyperBridge {
    /// 将 Hyper 服务端请求转换为 Vernal 请求。
    ///
    /// 取消令牌同时属于请求和 Body：连接适配器取消令牌后，即使 Hyper
    /// `Incoming` 暂时没有新帧，Body task 也会被唤醒并返回取消错误。
    #[must_use]
    pub fn from_hyper_request(
        request: Request<Incoming>,
        cancellation: CancellationToken,
    ) -> HttpRequest {
        let (parts, incoming) = request.into_parts();
        let frames = BodyStream::new(incoming).map(|frame| frame.map_err(HttpBodyError::transport));
        let body = HttpBody::from_stream(frames).with_cancellation(cancellation.clone());
        HttpRequest::new(Request::from_parts(parts, body), cancellation)
    }

    /// 将 Vernal 响应转换为 Hyper 可直接发送的标准响应。
    ///
    /// `HttpBody` 已实现标准 `http_body::Body`，因此响应侧无需 Box、复制或
    /// 二次适配，Hyper 会直接轮询其中的数据帧与 Trailer。
    #[must_use]
    pub fn into_hyper_response(response: HttpResponse) -> Response<HttpBody> {
        response.into_inner()
    }
}
