# vernal-websocket

> Vernal WebSocket 支持 — Spring Framework `spring-websocket` 的完整功能语义 Rust 迁移实现。

[![Crates.io](https://img.shields.io/crates/v/vernal-websocket)](https://crates.io/crates/vernal-websocket)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)

[English](#english) | [中文](#中文)

---

## English

### Overview

`vernal-websocket` provides a full-featured, async-first WebSocket implementation for Rust, faithfully porting the observable behavior of Spring Framework's `spring-websocket` module. It covers the complete WebSocket lifecycle — from handshake and session management to STOMP/SockJS compatibility — while using idiomatic Rust patterns (async traits, typed errors, bounded channels, explicit cancellation) instead of Java reflection, Servlet containers, or JVM-specific mechanisms.

**Key principle**: Public APIs never expose third-party crate types (`tokio_websockets::Message`, `tungstenite::Message`, etc.). All types are Vernal-owned, ensuring stable interfaces regardless of the underlying transport.

### Architecture

```
vernal-websocket (public API — no third-party types leaked)
├── WebSocketHandler / WebSocketSession / WebSocketMessage
├── handler decorators (12 types)
│   ├── AbstractWebSocketHandler, TextWebSocketHandler, BinaryWebSocketHandler
│   ├── WebSocketHandlerDecorator, WebSocketHandlerDecoratorFactory
│   ├── ExceptionWebSocketHandlerDecorator, LoggingWebSocketHandlerDecorator
│   ├── ConcurrentWebSocketSessionDecorator (flush-lock dual-lock semantics)
│   ├── PerConnectionWebSocketHandler, BeanCreatingHandlerProvider
│   └── WebSocketSessionDecorator, SessionLimitExceededError
├── server handshake chain (11 types)
│   ├── HandshakeHandler / HandshakeInterceptor / HandshakeInterceptorChain
│   ├── AbstractHandshakeHandler / DefaultHandshakeHandler
│   ├── OriginHandshakeInterceptor / HttpSessionHandshakeInterceptor
│   ├── WebSocketHandlerMapping / WebSocketHttpRequestHandler
│   └── RequestUpgradeStrategy / HandshakeFailureError
├── messaging / SubProtocol layer
│   ├── SubProtocolHandler / SubProtocolErrorHandler
│   ├── StompSubProtocolHandler (CONNECT→CONNECTED, DISCONNECT→RECEIPT)
│   ├── SubProtocolWebSocketHandler (session registry + protocol dispatch)
│   ├── WebSocketStompClient + StompSession state machine
│   ├── DefaultSimpUserRegistry (user/session/subscription CRUD)
│   ├── WebSocketAnnotationMethodMessageHandler (explicit destination router)
│   └── SessionConnectEvent / ConnectedEvent / DisconnectEvent / SubscribeEvent / UnsubscribeEvent
├── STOMP codec
│   ├── StompCommand (13 commands), StompHeaders, StompEncoder, StompDecoder
│   └── Header escaping per STOMP 1.2 spec
└── SockJS layer
    ├── frame: SockJsFrame, SockJsFrameType, SockJsMessageCodec, JsonSockJsMessageCodec
    ├── transport: TransportType (6 types), SockJsServiceConfig, TransportHandler SPI
    ├── session: AbstractSockJsSession, PollingSockJsSession, StreamingSockJsSession
    ├── handler: HttpReceivingTransportHandler, HttpSendingTransportHandler,
    │           WebSocketTransportHandler, SockJsWebSocketHandler, DefaultSockJsService
    └── client: SockJsClient (fallback chain), SockJsUrlInfo, InfoReceiver, Transport SPI
```

### Internal transport layer (not exposed to users)

| Backend | Version | MSRV | Status |
|---|---|---|---|
| `tokio-websockets` | 0.12.0 | 1.79 | **Default** — strict RFC 6455, `Bytes`-friendly |
| `tokio-tungstenite` | 0.30.0 | 1.85 | Compatible alternative |
| `fastwebsockets` | 0.10.0 | — | Optional high-performance |

External STOMP broker (recommended for production): **RabbitMQ** STOMP plugin / **ActiveMQ Artemis** STOMP.

### Quick start

```toml
[dependencies]
vernal-websocket = "0.0.0-dev"
tokio = { version = "1", features = ["full"] }
```

```rust
use vernal_websocket::{WebSocketHandler, WebSocketMessage, HandlerFuture, WebSocketError, WebSocketSession};

struct EchoHandler;

impl WebSocketHandler for EchoHandler {
    fn on_message<'a>(
        &'a self, session: &'a dyn WebSocketSession, message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            if let WebSocketMessage::Text(text) = message {
                session.send(WebSocketMessage::text(text)).await?;
            }
            Ok(())
        })
    }
}
```

### Testing

```bash
cargo test -p vernal-websocket          # 113 tests, all passing
cargo fmt --check -p vernal-websocket
cargo clippy -p vernal-websocket --all-targets
```

### Migration coverage (v4.0)

| Disposition | Count |
|---|---:|
| ✅ BEHAVIOR_VERIFIED | 84 |
| 🚫 JAVA_ONLY_EXEMPT | 54 |
| **Total** | **138** |

Four migration documents maintained in `docs/vernal-websocket/`: 迁移路线图, 对象级对照表, 语义迁移对照表, 对象名称一致性检查.

### Known limitations

- **permessage-deflate**: Not implemented (default OFF).
- **SockJS client**: Uses `HttpRequestExecutor` trait; provide your own HTTP client.
- **STOMP broker**: Codec + session + client only; use RabbitMQ/Artemis for production.
- **ConcurrentWebSocketSessionDecorator**: Uses `tokio::sync::Mutex::try_lock()` for non-blocking flush-lock.

---

## 中文

### 概述

`vernal-websocket` 是一个功能完整的、异步优先的 Rust WebSocket 实现，忠实复刻了 Spring Framework `spring-websocket` 模块的可观察行为。它覆盖完整的 WebSocket 生命周期——从握手、会话管理到 STOMP/SockJS 兼容——同时使用 Rust 惯用模式（async trait、类型化错误、有界通道、显式取消），而非 Java 反射、Servlet 容器或 JVM 特有机制。

**核心原则**：公共 API 不暴露任何第三方 crate 类型（`tokio_websockets::Message`、`tungstenite::Message` 等）。所有类型均为 Vernal 自有，确保接口稳定不受底层 transport 变更影响。

### 架构

```
vernal-websocket 公共 API（不暴露第三方类型）
├── WebSocketHandler / WebSocketSession / WebSocketMessage
├── handler 装饰器族（12 个对象）
│   ├── AbstractWebSocketHandler, TextWebSocketHandler, BinaryWebSocketHandler
│   ├── WebSocketHandlerDecorator, WebSocketHandlerDecoratorFactory
│   ├── ExceptionWebSocketHandlerDecorator, LoggingWebSocketHandlerDecorator
│   ├── ConcurrentWebSocketSessionDecorator（flush lock 双锁并发语义）
│   ├── PerConnectionWebSocketHandler, BeanCreatingHandlerProvider
│   └── WebSocketSessionDecorator, SessionLimitExceededError
├── server 握手链（11 个对象）
│   ├── HandshakeHandler / HandshakeInterceptor / HandshakeInterceptorChain
│   ├── AbstractHandshakeHandler / DefaultHandshakeHandler
│   ├── OriginHandshakeInterceptor / HttpSessionHandshakeInterceptor
│   ├── WebSocketHandlerMapping / WebSocketHttpRequestHandler
│   └── RequestUpgradeStrategy / HandshakeFailureError
├── messaging/SubProtocol 层
│   ├── SubProtocolHandler / SubProtocolErrorHandler
│   ├── StompSubProtocolHandler（CONNECT→CONNECTED、DISCONNECT→RECEIPT）
│   ├── SubProtocolWebSocketHandler（session 注册表 + 协议分派）
│   ├── WebSocketStompClient + StompSession 状态机
│   ├── DefaultSimpUserRegistry（用户/会话/订阅 CRUD）
│   ├── WebSocketAnnotationMethodMessageHandler（显式 destination 路由）
│   └── SessionConnectEvent / ConnectedEvent / DisconnectEvent / SubscribeEvent / UnsubscribeEvent
├── STOMP codec
│   ├── StompCommand（13 个命令）、StompHeaders、StompEncoder、StompDecoder
│   └── 符合 STOMP 1.2 规范的 header 转义（冒号/换行/反斜杠/回车）
└── SockJS 层
    ├── frame: SockJsFrame, SockJsFrameType, SockJsMessageCodec, JsonSockJsMessageCodec
    ├── transport: TransportType（6 种类型）、SockJsServiceConfig、TransportHandler SPI
    ├── session: AbstractSockJsSession, PollingSockJsSession（单帧 flush）
    │           StreamingSockJsSession（streamBytesLimit 回收）、WebSocketServerSockJsSession
    ├── handler: HttpReceivingTransportHandler（XHR POST）、HttpSendingTransportHandler（XHR/EventSource/HtmlFile）
    │           WebSocketTransportHandler、SockJsWebSocketHandler、DefaultSockJsService
    └── client: SockJsClient（transport 降级链）、SockJsUrlInfo、InfoReceiver、Transport SPI
                XhrTransportImpl、WebSocketClientTransport、DefaultTransportRequest、ClientSockJsSession
```

### 内部 transport 层（用户不直接接触）

| 后端 | 版本 | MSRV | 状态 |
|---|---|---|---|
| `tokio-websockets` | 0.12.0 | 1.79 | **默认** — 严格 RFC 6455、`Bytes` 友好 |
| `tokio-tungstenite` | 0.30.0 | 1.85 | 兼容备选 — 生态最广 |
| `fastwebsockets` | 0.10.0 | — | 可选高性能 |

外部 STOMP broker（生产推荐）：**RabbitMQ** STOMP 插件 / **ActiveMQ Artemis** STOMP。

### 快速开始

```toml
[dependencies]
vernal-websocket = "0.0.0-dev"
tokio = { version = "1", features = ["full"] }
```

```rust
use vernal_websocket::{WebSocketHandler, WebSocketMessage, HandlerFuture, WebSocketError, WebSocketSession};

struct EchoHandler;

impl WebSocketHandler for EchoHandler {
    fn on_message<'a>(
        &'a self, session: &'a dyn WebSocketSession, message: WebSocketMessage,
    ) -> HandlerFuture<'a, Result<(), WebSocketError>> {
        Box::pin(async move {
            if let WebSocketMessage::Text(text) = message {
                session.send(WebSocketMessage::text(text)).await?;
            }
            Ok(())
        })
    }
}
```

### 测试

```bash
cargo test -p vernal-websocket          # 113 个测试，全部通过
cargo fmt --check -p vernal-websocket
cargo clippy -p vernal-websocket --all-targets
```

### 迁移覆盖（v4.0）

| 处置方式 | 数量 |
|---|---:|
| ✅ BEHAVIOR_VERIFIED（实现且测试覆盖） | 84 |
| 🚫 JAVA_ONLY_EXEMPT（JVM/Servlet/Jetty 特有，批准豁免） | 54 |
| **合计** | **138** |

四份迁移文档维护在 `docs/vernal-websocket/`：迁移路线图、对象级对照表、语义迁移对照表、对象名称一致性检查。

### 已知限制

- **permessage-deflate**：未实现（默认关闭）。
- **SockJS client**：使用 `HttpRequestExecutor` trait，需自行提供 HTTP 客户端实现。
- **STOMP broker**：仅提供 codec + session + client，不内置内存 broker；生产请用 RabbitMQ/Artemis。
- **ConcurrentWebSocketSessionDecorator**：使用 `tokio::sync::Mutex::try_lock()` 实现非阻塞 flush lock 语义。

### License

MIT
