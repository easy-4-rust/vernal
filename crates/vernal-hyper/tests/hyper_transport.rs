//! Hyper 真实网络传输合同测试。

use std::{convert::Infallible, io};

use http::{Response, header};
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_util::sync::CancellationToken;
use vernal_http::{HttpBody, HttpResponse};
use vernal_hyper::HyperBridge;

/// 在真实 TCP 连接上验证请求帧、Trailer 和响应发送过程。
#[tokio::test]
async fn hyper_bridge_preserves_wire_request_and_trailers() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let address = listener.local_addr().expect("address should exist");

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("connection should arrive");
        let service = service_fn(|request| async move {
            let request = HyperBridge::from_hyper_request(request, CancellationToken::new());
            let snapshot = request.snapshot();
            let (_, body, _) = request.into_parts();
            let collected = body
                .collect_limited(64)
                .await
                .expect("request body should collect");
            let checksum = collected
                .trailers()
                .and_then(|trailers| trailers.get("x-checksum"))
                .and_then(|value| value.to_str().ok())
                .unwrap_or("missing");
            let text = format!(
                "{} {} {} {checksum}",
                snapshot.method(),
                snapshot.uri().path(),
                String::from_utf8_lossy(collected.bytes())
            );
            let response = Response::builder()
                .status(200)
                .header(header::CONNECTION, "close")
                .body(HttpBody::full(text))
                .expect("response should build");
            Ok::<_, Infallible>(HyperBridge::into_hyper_response(HttpResponse::new(
                response,
            )))
        });

        http1::Builder::new()
            .serve_connection(TokioIo::new(stream), service)
            .await
            .expect("HTTP connection should complete");
    });

    let response = send_chunked_request(address)
        .await
        .expect("wire request should complete");
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.ends_with("POST /upload hello sha256:abc"));
    server.await.expect("server task should join");
}

/// 使用原始 HTTP/1.1 chunked 编码发送包含 Trailer 的真实请求。
async fn send_chunked_request(address: std::net::SocketAddr) -> io::Result<String> {
    let mut stream = TcpStream::connect(address).await?;
    stream
        .write_all(
            b"POST /upload HTTP/1.1\r\n\
              Host: localhost\r\n\
              Transfer-Encoding: chunked\r\n\
              Trailer: x-checksum\r\n\
              Connection: close\r\n\
              \r\n\
              5\r\nhello\r\n\
              0\r\nx-checksum: sha256:abc\r\n\r\n",
        )
        .await?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;
    Ok(String::from_utf8_lossy(&response).into_owned())
}
