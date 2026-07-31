<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-messaging 技术要求
> 迁移文档治理：本文级别为 **authoritative**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。


> 当前权威要求；遵循[迁移验收规范](../迁移验收规范.md)，Spring 基线为 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。

## 当前事实

[自动对象审计](../migration-audit/vernal-messaging.md)识别 222 个业务对象：0 个严格完成、2 个 `MISPLACED`、219 个 `MISSING`、1 个 `UNVERIFIED`。现有消息和通道骨架不能代表 converter、template、handler、SIMP、STOMP、RSocket 与 TCP 已迁移。

## 目标结构

| Spring 来源 | 目标 Rust |
|---|---|
| `support/AbstractMessageChannel.java` | `support/abstract_message_channel.rs` |
| `handler/invocation/AbstractMethodMessageHandler.java` | `handler/invocation/abstract_method_message_handler.rs` |
| `simp/broker/AbstractBrokerMessageHandler.java` | `simp/broker/abstract_broker_message_handler.rs` |
| `simp/stomp/StompDecoder.java` | `simp/stomp/stomp_decoder.rs` |
| `rsocket/service/RSocketServiceMethod.java` | `rsocket/service/r_socket_service_method.rs` |

只保留末两层包目录；一对象一文件；`mod.rs`、`lib.rs` 只声明和重导出。

## 语义要求

- 消息必须保持 payload、只读 headers、ID/timestamp 与错误消息追踪语义。
- 通道必须保持 interceptor 的 pre/post/completion 顺序及倒序异常完成。
- 订阅通道必须保持同步/异步执行、拒绝回退、每个 handler 的 before/after。
- 模板、转换器、目标解析与注解方法调用必须保持超时、类型转换和异常包装。
- 协议 companion crate 不是对象豁免；只有精确依赖符号和集成测试可标 `DEPENDENCY_REUSED`。

---

<!-- restored-detail-from-head: dd20300d16a09200bd8a379ff14db1e2da99b67c -->

## 原详细文档（完整保留）

> 以下正文完整恢复自 Vernal 提交 `dd20300d16a09200bd8a379ff14db1e2da99b67c`。其中历史对象数量、完成状态、
> 路径算法和依赖替代结论如与本文顶部或自动对象台账冲突，以顶部当前结论和
> `docs/migration-audit/` 为准；其 API、设计背景、阶段拆解和测试说明继续保留。

# vernal-messaging 技术要求（对标 spring-messaging）

> **版本**：v1.0（2026-07-28）
> **定位**：vernal-messaging crate 技术交接文档，对标 Spring Framework 7.0.8 spring-messaging。
> **主线**：vernal 只提供抽象 trait，broker adapter 全部在 ddd4r 实现。
> **现状**：6 文件 / 449 行，edition 2024 / rustc 1.88。
> **引用约定**：crate 选型依据见《Spring 组件替换约定》第五节。

---

## 一、总览

### 1.1 定位与边界

vernal-messaging 是 Vernal Framework 的 **消息通道抽象层**，
对标 spring-messaging 模块，提供 Message / Channel / Handler 核心抽象。

| 维度 | spring-messaging（语义参考） | vernal-messaging（实现） | 差异说明 |
|:---|:---|:---|:---|
| 语言 | Java（泛型 + 接口） | Rust（trait + Arc） | 零拷贝共享语义 |
| 异步 | Reactor Mono/Flux / 阻塞 | Tokio-first async | 原生异步，无桥接 |
| 消息载体 | `Message<T>` 泛型 | `Message` trait（`&[u8]` 负载） | 字节负载，零序列化约束 |
| 通道模型 | `MessageChannel` / `SubscribableChannel` / `PollableChannel` | 同名 trait | 语义对齐 |
| Broker 集成 | 内嵌 STOMP / WebSocket | 抽象 trait，broker 在 ddd4r | 职责分离 |
| JMS 语义 | 独立 spring-jms 模块 | 融入 2.11 节（不新建 jms crate） | 简化架构 |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────┐
│  应用层：ddd4r broker adapter（STOMP / AMQP / MQTT） │
├─────────────────────────────────────────────────────┤
│  用户层：MessageChannel / SubscribableChannel        │
│  → send / receive / subscribe / unsubscribe         │
├─────────────────────────────────────────────────────┤
│  处理层：MessageHandler trait                         │
│  → handle_message(Arc<dyn Message>)                  │
├─────────────────────────────────────────────────────┤
│  消息层：Message trait / GenericMessage               │
│  → id + payload(&[u8]) + headers(BTreeMap)           │
├─────────────────────────────────────────────────────┤
│  SIMP 层：SimpMessageType / SimpMessageHeaderAccessor │
│  → DefaultSimpUserRegistry（用户/会话/订阅管理）      │
└─────────────────────────────────────────────────────┘
```

### 1.3 关键决策

| 项 | 决策 | 理由 |
|:---|:---|:---|
| 负载类型 | `&[u8]` 字节切片 | 零序列化约束，上层按需解码 |
| 头部存储 | `BTreeMap<String, String>` | 确定性排序，满足 STOMP 头语义 |
| 泛型 vs trait object | `dyn Message` trait object | 消息异构，编译期无法统一类型 |
| Broker 职责 | vernal 只抽象，ddd4r 实现 | 框架与业务分离 |
| JMS 语义 | 融入 messaging 而非独立 crate | JMS 是通道语义子集，不值得独立 crate |

### 1.4 命名映射

| Spring 原名 | vernal-messaging 移植名 | 说明 |
|:---|:---|:---|
| `Message<T>` | `Message` trait | 泛型 → trait object |
| `GenericMessage<T>` | `GenericMessage` | `T` → `Vec<u8>` |
| `MessageHeaders` | `BTreeMap<String, String>` | 内联到 Message trait |
| `MessageChannel` | `MessageChannel` trait | 直接对标 |
| `SubscribableChannel` | `SubscribableChannel` trait | 直接对标 |
| `PollableChannel` | `PollableChannel` trait（待建） | 直接对标 |
| `MessageHandler` | `MessageHandler` trait | 直接对标 |
| `MessageConverter` | `MessageConverter` trait（待建） | 直接对标 |
| `SimpMessageHeaderAccessor` | `SimpMessageHeaderAccessor` | 直接对标 |
| `SimpMessageType` | `SimpMessageType` enum | 直接对标 |
| `DefaultSimpUserRegistry` | `DefaultSimpUserRegistry` | 直接对标 |

---

## 二、核心 Trait 体系

### 2.1 Message trait —— 消息载体契约

**来源**：vernal-messaging 现有实现。
**语义参照**：spring-messaging `Message<T>`。

#### Spring API（Java）

```java
// spring-messaging 核心消息接口
public interface Message<T> {
    T getPayload();
    MessageHeaders getHeaders();
}
```

#### Rust trait

```rust
/// 消息 trait。对标 Spring `Message<T>`。
/// 负载统一为 `&[u8]`，上层按需解码。
pub trait Message: Send + Sync {
    /// 消息 ID。对标 MessageHeaders#ID。
    fn id(&self) -> &str;

    /// 消息负载（字节切片）。对标 getPayload()。
    fn payload(&self) -> &[u8];

    /// 消息头。对标 getHeaders()。
    /// 默认返回空 BTreeMap；自定义实现可覆盖。
    fn headers(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享，可存入 Arc |
| 负载类型 | `&[u8]` 字节切片 | 零序列化约束 |
| 头部类型 | `BTreeMap<String, String>` | 确定性排序，STOMP 头兼容 |
| ID 类型 | `&str` | 轻量引用，避免分配 |

#### 待补齐

- [x] `Message` trait 定义 + 默认 `headers()` 实现
- [x] `GenericMessage` 具体实现
- [ ] 泛型版本 `TypedMessage<T: Serialize + Deserialize>`（P2）

---

### 2.2 GenericMessage —— 通用字节消息

**来源**：vernal-messaging 现有实现。
**语义参照**：spring-messaging `GenericMessage<T>`。

#### Spring API（Java）

```java
// spring-messaging 通用消息实现
public class GenericMessage<T> implements Message<T> {
    public GenericMessage(T payload) { ... }
    public GenericMessage(T payload, MessageHeaders headers) { ... }
}
```

#### Rust 实现

```rust
/// 通用字节消息实现。对标 Spring `GenericMessage<T>`。
#[derive(Debug, Clone)]
pub struct GenericMessage {
    /// 消息 ID。
    pub message_id: String,
    /// 负载（字节）。
    pub data: Vec<u8>,
    /// 消息头。
    pub header_map: BTreeMap<String, String>,
}

impl GenericMessage {
    /// 创建消息。
    #[must_use]
    pub fn new(id: impl Into<String>, data: Vec<u8>) -> Self;

    /// 链式添加头。
    #[must_use]
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self;
}

impl Message for GenericMessage { ... }
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Clone` | 必须 | 消息可复制，支持多消费者 |
| `Debug` | 必须 | 调试日志 |
| ID 生成 | 由调用方提供 | vernal 不内置 UUID 生成 |

#### 待补齐

- [x] `GenericMessage` 实现 + `Message` trait 实现
- [ ] `TypedMessage<T>` 泛型消息（P2）

---

### 2.3 MessageChannel trait —— 消息通道契约

**来源**：vernal-messaging 现有实现。
**语义参照**：spring-messaging `MessageChannel`。

#### Spring API（Java）

```java
// spring-messaging 消息通道接口
public interface MessageChannel {
    boolean send(Message<?> message);
    boolean send(Message<?> message, long timeout);
}
```

#### Rust trait

```rust
/// 消息通道 trait。对标 Spring `MessageChannel`。
/// 使用 Rust 异步 Future 表达发送/接收。
pub trait MessageChannel: Send + Sync {
    /// 异步发送消息。对标 send()。
    fn send<'a>(&'a self, message: Arc<dyn Message>) -> SendFuture<'a>;

    /// 异步接收消息（非阻塞语义：无消息返回 Ok(None)）。
    /// 对标 PollableChannel.receive()。
    fn receive<'a>(&'a self) -> ReceiveFuture<'a> {
        Box::pin(async { Ok(None) })
    }
}
```

#### Future 类型

```rust
/// 发送 Future：完成即确认。
pub type SendFuture<'a> = Pin<Box<dyn Future<Output = Result<(), MessageError>> + Send + 'a>>;

/// 接收 Future：返回 Option（无消息时 None）。
pub type ReceiveFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<Arc<dyn Message>>, MessageError>> + Send + 'a>>;
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享 |
| 消息类型 | `Arc<dyn Message>` | 共享所有权，支持多消费者 |
| 超时 | 由调用方通过 `tokio::time::timeout` 包裹 | vernal 不内置超时语义 |

#### 待补齐

- [x] `MessageChannel` trait 定义
- [x] `SendFuture` / `ReceiveFuture` 类型别名
- [ ] `PollableChannel` trait（待建，显式区分可轮询通道）

---

### 2.4 SubscribableChannel trait —— 可订阅通道

**来源**：vernal-messaging 现有实现。
**语义参照**：spring-messaging `SubscribableChannel`。

#### Spring API（Java）

```java
// spring-messaging 可订阅通道
public interface SubscribableChannel extends MessageChannel {
    boolean subscribe(MessageHandler<?> handler);
    boolean unsubscribe(MessageHandler<?> handler);
}
```

#### Rust trait

```rust
/// 可订阅消息通道 trait。对标 Spring `SubscribableChannel`。
pub trait SubscribableChannel: MessageChannel {
    /// 订阅消息处理器；返回订阅句柄。
    fn subscribe(&self, handler: Arc<dyn MessageHandler>) -> SubscriptionId;

    /// 取消订阅。
    fn unsubscribe(&self, id: SubscriptionId);
}

/// 订阅 ID（句柄）。对标 Spring handler 引用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| 继承 | `MessageChannel` | 可订阅通道首先是消息通道 |
| 订阅句柄 | `SubscriptionId`（u64） | 轻量句柄，支持精确取消 |
| Handler 存储 | 由实现决定 | `InMemoryChannel` 用内部 Vec |

#### 待补齐

- [x] `SubscribableChannel` trait 定义
- [x] `SubscriptionId` 句柄类型

---

## 三、消息处理与转换

### 3.1 MessageHandler trait —— 消息处理器

**来源**：vernal-messaging 现有实现。
**语义参照**：spring-messaging `MessageHandler<T>`。

#### Spring API（Java）

```java
// spring-messaging 消息处理器
public interface MessageHandler<T> {
    void handleMessage(Message<? extends T> message) throws MessagingException;
}
```

#### Rust trait

```rust
/// 消息处理器 trait。对标 Spring `MessageHandler<T>`。
pub trait MessageHandler: Send + Sync {
    /// 处理消息。对标 handleMessage()。
    fn handle_message<'a>(&'a self, message: Arc<dyn Message>) -> SendFuture<'a>;
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享 |
| 返回类型 | `SendFuture<'a>` | 异步处理 |
| 错误语义 | `MessageError` | 统一错误类型 |

#### 待补齐

- [x] `MessageHandler` trait 定义
- [ ] `AsyncMessageHandler`（带 `async fn` 版本，P2）

---

### 3.2 MessageConverter trait —— 消息转换器（待建）

**语义参照**：spring-messaging `MessageConverter`。

#### Spring API（Java）

```java
// spring-messaging 消息转换器
public interface MessageConverter {
    Message<?> toMessage(Object object, MessageHeaders headers);
    Object fromMessage(Message<?> message, Class<?> targetClass);
}
```

#### Rust trait（设计稿）

```rust
/// 消息转换器 trait。对标 Spring `MessageConverter`。
/// 在字节负载与领域类型之间双向转换。
pub trait MessageConverter: Send + Sync {
    /// 领域对象 → 字节消息。
    fn to_message(
        &self,
        payload: &[u8],
        headers: BTreeMap<String, String>,
    ) -> Result<GenericMessage, MessageError>;

    /// 字节消息 → 领域对象（返回字节切片，由调用方反序列化）。
    fn from_message(
        &self,
        message: &dyn Message,
    ) -> Result<Vec<u8>, MessageError>;
}
```

#### 待补齐

- [ ] `MessageConverter` trait 定义（P1）
- [ ] `ByteArrayMessageConverter` 默认实现（P1）
- [ ] `MappingJackson2MessageConverter` → serde_json 替代（P2）

---

### 3.3 MessageError —— 消息错误

**来源**：vernal-messaging 现有实现。

```rust
/// 消息错误。对标 Spring `MessagingException`。
#[derive(Debug, Clone)]
pub struct MessageError {
    pub message: String,
}

impl Display for MessageError { ... }
impl Error for MessageError {}
```

---

## 四、SIMP 子系统（STOMP 简化语义）

### 4.1 SimpMessageType —— SIMP 消息类型

**来源**：vernal-messaging 现有实现。
**语义参照**：spring-messaging `SimpMessageType`。

#### Spring API（Java）

```java
// spring-messaging SIMP 消息类型枚举
public enum SimpMessageType {
    CONNECT, DISCONNECT, SUBSCRIBE, UNSUBSCRIBE,
    MESSAGE, HEARTBEAT, OTHER
}
```

#### Rust enum

```rust
/// SIMP 消息类型。对标 Spring `SimpMessageType`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpMessageType {
    Connect,      // CONNECT/STOMP
    Disconnect,   // DISCONNECT
    Subscribe,    // SUBSCRIBE
    Unsubscribe,  // UNSUBSCRIBE
    Message,      // SEND/MESSAGE
    Heartbeat,    // HEARTBEAT
    Other,        // 其它
}

impl SimpMessageType {
    /// 返回 Spring header 字符串值。
    #[must_use]
    pub const fn as_str(self) -> &'static str;
}
```

---

### 4.2 SimpMessageHeaderAccessor —— SIMP 消息头访问器

**来源**：vernal-messaging 现有实现。
**语义参照**：spring-messaging `SimpMessageHeaderAccessor` + `SimpAttributes`。

#### Spring API（Java）

```java
// spring-messaging SIMP 头访问器
public class SimpMessageHeaderAccessor extends NativeMessageHeaderAccessor {
    public static final String SIMP_MESSAGE_TYPE = "simpMessageType";
    public static final String SIMP_SESSION_ID = "simpSessionId";
    // ... 更多常量
}
```

#### Rust 实现

```rust
/// SIMP 消息头访问器。
#[derive(Debug, Clone, Default)]
pub struct SimpMessageHeaderAccessor {
    headers: BTreeMap<String, String>,
}

impl SimpMessageHeaderAccessor {
    pub fn new() -> Self;
    pub fn from_headers(headers: BTreeMap<String, String>) -> Self;
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>);
    pub fn get(&self, name: &str) -> Option<&str>;
    pub fn headers(&self) -> &BTreeMap<String, String>;
    pub fn session_id(&self) -> Option<&str>;
    pub fn destination(&self) -> Option<&str>;
    pub fn subscription_id(&self) -> Option<&str>;
    pub fn user(&self) -> Option<&str>;
}
```

#### simp_headers 常量模块

```rust
pub mod simp_headers {
    pub const SIMP_MESSAGE_TYPE: &str = "simpMessageType";
    pub const SIMP_SESSION_ID: &str = "simpSessionId";
    pub const SIMP_SUBSCRIPTION_ID: &str = "simpSubscriptionId";
    pub const SIMP_DESTINATION: &str = "simpDestination";
    pub const SIMP_USER: &str = "simpUser";
    pub const SIMP_CONNECT_MESSAGE: &str = "simpConnectMessage";
    pub const SIMP_DISCONNECT_MESSAGE: &str = "simpDisconnectMessage";
    pub const SIMP_HEARTBEAT: &str = "simpHeartbeat";
    pub const SIMP_ORIG_DESTINATION: &str = "simpOrigDestination";
    pub const SIMP_IGNORE_ERROR: &str = "simpIgnoreError";
    pub const SIMP_SESSION_ATTRIBUTES: &str = "simpSessionAttributes";
}
```

---

### 4.3 DefaultSimpUserRegistry —— 用户/会话/订阅注册表

**来源**：vernal-messaging 现有实现。
**语义参照**：spring-messaging `DefaultSimpUserRegistry` / `SimpUser` / `SimpSession` / `SimpSubscription`。

#### Spring API（Java）

```java
// spring-messaging SIMP 用户注册表
public interface SimpUserRegistry {
    Collection<? extends SimpUser> getUsers();
    SimpUser getUser(String name);
}
```

#### Rust 实现

```rust
/// SIMP 订阅。对标 Spring `SimpSubscription`。
#[derive(Debug, Clone)]
pub struct SimpSubscription {
    pub id: String,
    pub destination: String,
    pub session_id: String,
}

/// SIMP session。对标 Spring `SimpSession`。
#[derive(Debug, Clone, Default)]
pub struct SimpSession {
    pub id: String,
    pub user: Option<String>,
    pub subscriptions: BTreeMap<String, SimpSubscription>,
}

/// SIMP user。对标 Spring `SimpUser`。
#[derive(Debug, Clone, Default)]
pub struct SimpUser {
    pub name: String,
    pub sessions: BTreeMap<String, SimpSession>,
}

/// 默认 SIMP 用户注册表。对标 Spring `DefaultSimpUserRegistry`。
/// 内部使用 `RwLock` 保证并发安全。
#[derive(Debug, Default)]
pub struct DefaultSimpUserRegistry {
    users: RwLock<BTreeMap<String, SimpUser>>,
}

impl DefaultSimpUserRegistry {
    pub fn new() -> Self;
    pub async fn register_session(&self, user: Option<String>, session_id: impl Into<String>);
    pub async fn remove_session(&self, session_id: &str);
    pub async fn add_subscription(&self, session_id: &str, subscription: SimpSubscription);
    pub async fn remove_subscription(&self, session_id: &str, subscription_id: &str);
    pub async fn get_user(&self, name: &str) -> Option<SimpUser>;
    pub async fn user_names(&self) -> Vec<String>;
    pub async fn get_session(&self, session_id: &str) -> Option<SimpSession>;
    pub async fn user_count(&self) -> usize;
    pub async fn session_count(&self) -> usize;
}

/// 共享句柄。对标 Spring 注入的 `SimpUserRegistry` 单例。
pub type SharedSimpUserRegistry = Arc<DefaultSimpUserRegistry>;
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| 并发安全 | `RwLock` | 读多写少场景 |
| 共享方式 | `Arc<DefaultSimpUserRegistry>` | 跨组件共享 |
| 异步 | 所有方法均为 `async` | Tokio 兼容 |

---

## 五、JMS 语义融入（不新建 jms crate）

### 5.1 设计决策

Spring 独立 `spring-jms` 模块提供 JMS 抽象。Vernal 决定 **不新建 jms crate**，
将 JMS 语义融入 vernal-messaging 的 trait 体系中。理由：

| 维度 | Spring 方案 | Vernal 方案 | 决策理由 |
|:---|:---|:---|:---|
| 模块独立性 | spring-jms 独立 crate | 融入 messaging | JMS 是通道语义子集 |
| 连接工厂 | `ConnectionFactory` | 由 ddd4r broker adapter 实现 | vernal 只抽象 |
| 会话管理 | `Session` / `JmsTemplate` | 由 ddd4r 实现 | vernal 只抽象 |
| 消息类型 | `TextMessage` / `BytesMessage` / `MapMessage` | `Message` trait（`&[u8]`） | 统一字节负载 |

### 5.2 JMS 概念在 vernal-messaging 中的承载

| JMS 概念 | Spring JMS 类 | vernal-messaging 承载 | 说明 |
|:---|:---|:---|:---|
| 连接工厂 | `ConnectionFactory` | ddd4r broker adapter | vernal 不抽象连接 |
| 会话 | `Session` | ddd4r broker adapter | vernal 不抽象会话 |
| 消息生产者 | `MessageProducer` | `MessageChannel::send()` | 统一发送语义 |
| 消息消费者 | `MessageConsumer` | `SubscribableChannel::subscribe()` | 统一订阅语义 |
| 队列 | `Queue` | `MessageChannel` | 通道即队列 |
| 主题 | `Topic` | `SubscribableChannel` | 订阅即主题 |
| 点对点 | `JmsTemplate.receive()` | `MessageChannel::receive()` | 统一接收语义 |
| 发布订阅 | `@JmsListener` | `MessageHandler::handle_message()` | 统一处理语义 |

### 5.3 JMS 消息类型映射

| JMS 消息类型 | vernal-messaging 映射 | 说明 |
|:---|:---|:---|
| `TextMessage` | `GenericMessage`（UTF-8 字节） | 文本即字节 |
| `BytesMessage` | `GenericMessage`（原始字节） | 原生字节 |
| `MapMessage` | `GenericMessage` + JSON 序列化 | 由调用方序列化 |
| `ObjectMessage` | `GenericMessage` + serde 序列化 | 由调用方序列化 |
| `StreamMessage` | `GenericMessage`（流式字节） | 由调用方处理 |

### 5.4 ddd4r broker adapter 接口约定

vernal-messaging 定义抽象 trait，ddd4r 实现具体 broker adapter：

```rust
// ddd4r 中的 broker adapter 实现模式（不在 vernal-messaging 中）
pub struct StompBrokerAdapter { /* STOMP 连接细节 */ }
impl MessageChannel for StompBrokerAdapter { ... }
impl SubscribableChannel for StompBrokerAdapter { ... }

pub struct AmqpBrokerAdapter { /* AMQP 连接细节 */ }
impl MessageChannel for AmqpBrokerAdapter { ... }
impl SubscribableChannel for AmqpBrokerAdapter { ... }
```

---

## 六、运行时与集成约束

### 6.1 运行时核心对象

| 对象 | 说明 | spring-messaging 对应 |
|:---|:---|:---|
| `Message` trait | 消息载体契约（id + payload + headers） | `Message<T>` |
| `GenericMessage` | 通用字节消息实现 | `GenericMessage<T>` |
| `MessageChannel` trait | 消息通道契约（send + receive） | `MessageChannel` |
| `SubscribableChannel` trait | 可订阅通道（subscribe + unsubscribe） | `SubscribableChannel` |
| `MessageHandler` trait | 消息处理器契约 | `MessageHandler<T>` |
| `InMemoryChannel` | 内存有界通道实现（mpsc） | （无直接对应） |
| `SubscriptionId` | 订阅句柄 | handler 引用 |
| `SimpMessageType` | SIMP 消息类型枚举 | `SimpMessageType` |
| `SimpMessageHeaderAccessor` | SIMP 头访问器 | `SimpMessageHeaderAccessor` |
| `DefaultSimpUserRegistry` | 用户/会话/订阅注册表 | `DefaultSimpUserRegistry` |

### 6.2 与 spring-messaging 概念的不移植项

| spring-messaging 概念 | 不移植原因 | vernal-messaging 替代 |
|:---|:---|:---|
| `GenericMessage<T>` 泛型 | Rust trait object 更适合异构消息 | `Message` trait + `GenericMessage` 字节 |
| `MessageBuilder` | Rust 构造器模式（`with_header`） | `GenericMessage::new().with_header()` |
| `MessageChannelInterceptor` | 由 vernal-aop `Interceptor` 承载 | 统一 AOP 拦截 |
| `MessageSendingTemplate` | ddd4r 实现 | vernal 只抽象 |
| `ReactiveMessageHandler` | Tokio 原生异步 | `MessageHandler` 已是异步 |
| `WebSocket` 子系统 | ddd4r 实现 | vernal 只抽象 |

### 6.3 引用约定文档

crate 选型依据、集成模式和硬约束见《Spring 组件替换约定》：

- **第五节**：vernal-messaging 对标 spring-messaging，vernal-macros 对标
  过程宏，vernal-context-indexer 对标 linkme 分布式 slice 注册。

### 6.4 待补齐工作路线图

| 阶段 | 内容 | 优先级 | 预估工作量 |
|:---|:---|:---|:---|
| P1 | `PollableChannel` trait 分离 | P1 | 0.5 天 |
| P1 | `MessageConverter` trait + `ByteArrayMessageConverter` | P1 | 1 天 |
| P2 | `TypedMessage<T>` 泛型消息 | P2 | 1 天 |
| P2 | `MessageBuilder` 构造器 | P2 | 0.5 天 |
| ddd4r | STOMP broker adapter | P0 | 3 天 |
| ddd4r | AMQP broker adapter | P1 | 2 天 |

#### PollableChannel trait（设计稿）

```rust
/// 可轮询消息通道。对标 Spring `PollableChannel`。
/// 从 `MessageChannel::receive()` 分离而来，显式区分阻塞/非阻塞语义。
pub trait PollableChannel: MessageChannel {
    /// 阻塞接收（带超时）。
    fn receive_with_timeout<'a>(
        &'a self,
        timeout: Duration,
    ) -> ReceiveFuture<'a>;
}
```

### 6.5 测试基线

#### 已完成

| 测试内容 | 数量 | 说明 |
|:---|:---|:---|
| `Message` trait + `GenericMessage` | 4 | 创建、头设置、负载访问 |
| `MessageChannel` + `InMemoryChannel` | 3 | 发送、接收、关闭 |
| `SubscribableChannel` | 2 | 订阅、取消 |
| `SimpMessageType` | 7 | 所有变体 + `as_str()` |
| `SimpMessageHeaderAccessor` | 5 | 创建、设置、获取、便捷方法 |
| `DefaultSimpUserRegistry` | 8 | 注册、移除、订阅、查询 |
| 总计 | 29 | |

#### 待做

| 测试内容 | 预估数量 |
|:---|:---|
| `PollableChannel` | 3+ |
| `MessageConverter` | 4+ |
| ddd4r STOMP adapter 集成 | 10+ |

### 6.6 成熟度状态

| 维度 | 当前 | 目标 |
|:---|:---|:---|
| 文件数 | 6 | 10+ |
| 行数 | 449 | 800+ |
| 核心 trait | 4（Message / Channel / Subscribable / Handler） | 6（+ Pollable / Converter） |
| SIMP 类型 | 3（MessageType / HeaderAccessor / UserRegistry） | 3 |
| Broker adapter | 0（在 ddd4r） | 3+（STOMP / AMQP / MQTT） |
| 与 spring-messaging 语义对标度 | ~60% | 80%+ |
| 测试数 | 29 | 50+ |

---

## 附录：spring-messaging 语义覆盖全景

| spring-messaging 包 | 类数 | vernal-messaging 状态 | 说明 |
|:---|:---|:---|:---|
| 根包（Message/Channel/Handler） | 15 | ✅ 已移植 | 核心 trait 体系 |
| simp（SIMP 协议） | 12 | ✅ 已移植 | SimpMessageType / HeaderAccessor / UserRegistry |
| support（消息支持） | 20 | ⬜ 部分 | MessageConverter 待建 |
| jms（JMS 语义） | 25 | 📎 融入 2.11 | 不新建 jms crate |
| reactive（响应式） | 8 | 🚫 不迁移 | Tokio 原生异步替代 |
| websocket（WebSocket） | 15 | 🚫 ddd4r 实现 | vernal 只抽象 |
| stomp（STOMP 协议） | 20 | 🚫 ddd4r 实现 | vernal 只抽象 |
| codec（编解码） | 10 | ⬜ 待定 | 由上层 serde 处理 |
