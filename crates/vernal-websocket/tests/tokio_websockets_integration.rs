//! 对标 Spring `spring-websocket` 集成测试：使用 `tokio-websockets` 通过本地 TCP
//! 建立真实 WebSocket 连接，验证 Vernal 消息 ↔ tokio-websockets 消息转换。

#![cfg(feature = "tokio-websockets")]

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_websockets::{ClientBuilder, ServerBuilder};
use vernal_websocket::{
    CloseCode, CloseStatus, WebSocketMessage, from_tokio_message, to_tokio_message,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn text_message_round_trips_between_vernal_and_tokio_websockets() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (_req, mut ws) = ServerBuilder::new().accept(stream).await.unwrap();
        while let Some(msg) = ws.next().await {
            let msg = msg.unwrap();
            if msg.is_text() {
                let echoed = tokio_websockets::Message::text("echo: hello");
                ws.send(echoed).await.unwrap();
                break;
            }
        }
    });

    let client_stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let (mut client, _resp) = ClientBuilder::new()
        .uri(&format!("ws://127.0.0.1:{port}"))
        .unwrap()
        .connect_on(client_stream)
        .await
        .unwrap();

    // Vernal -> tokio-websockets：发送 "hello"。
    let vernal_out = WebSocketMessage::text("hello");
    client
        .send(to_tokio_message(vernal_out).unwrap())
        .await
        .unwrap();

    // 服务端回送 "echo: hello"，转换为 Vernal 消息。
    let reply = client.next().await.unwrap().unwrap();
    let vernal_in = from_tokio_message(reply).unwrap();
    match vernal_in {
        WebSocketMessage::Text(text) => assert_eq!(text, "echo: hello"),
        other => panic!("期望 Text 消息，实际为 {other:?}"),
    }

    server.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_status_converts_to_tokio_websockets_close_frame() {
    let status = CloseStatus::new(CloseCode::Normal, "done").unwrap();
    let close_msg = WebSocketMessage::Close(Some(status));
    let tokio_msg = to_tokio_message(close_msg).unwrap();
    assert!(tokio_msg.is_close());
    if let Some((code, reason)) = tokio_msg.as_close() {
        assert_eq!(u16::from(code), 1000);
        assert_eq!(reason, "done");
    } else {
        panic!("预期 close frame");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn binary_payload_survives_round_trip() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (_req, mut ws) = ServerBuilder::new().accept(stream).await.unwrap();
        while let Some(msg) = ws.next().await {
            let msg = msg.unwrap();
            if msg.is_binary() {
                ws.send(msg).await.unwrap();
                break;
            }
        }
    });

    let client_stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let (mut client, _resp) = ClientBuilder::new()
        .uri(&format!("ws://127.0.0.1:{port}"))
        .unwrap()
        .connect_on(client_stream)
        .await
        .unwrap();

    let payload = Bytes::from_static(b"\x00\x01\x02\xFF");
    client
        .send(to_tokio_message(WebSocketMessage::binary(payload.clone())).unwrap())
        .await
        .unwrap();
    let reply = client.next().await.unwrap().unwrap();
    let vernal_in = from_tokio_message(reply).unwrap();
    match vernal_in {
        WebSocketMessage::Binary(received) => assert_eq!(received, payload),
        other => panic!("期望 Binary 消息，实际为 {other:?}"),
    }

    server.await.unwrap();
}
