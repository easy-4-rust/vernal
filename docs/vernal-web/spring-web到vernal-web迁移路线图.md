<!-- migration-doc: authority=historical canonical=迁移路线图.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [迁移路线图.md](迁移路线图.md) 为准。

> **历史文档，非当前验收依据。** 当前版本见[迁移路线图](迁移路线图.md)。

# spring-web → vernal-web 全量迁移路线图

> 版本：v1.0（2026-07-27）｜基线：Spring Framework 7.0.8 `spring-web`（含 `org.springframework.http`、`http.client`、`http.codec`、`http.converter`、`web` 公共契约）
> 目标：在 Rust 中实现 Web 基础组件的**功能语义完全迁移**；实现方式采用 Rust/异步生态惯用设计，不复制 Java 继承树或 Servlet 专有机制。
> 依据：`vernal-expression` 的迁移规范、对象一文件、snake_case/PascalCase、中文文档与验收清单约定。

## 一、CodeGraph 基线与当前事实

| 项目 | CodeGraph/源码事实 |
|---|---|
| `vernal-framework` 图谱 | 4,606 节点、34,731 边、754 文件；主要社区为 `src-application`、`src-trait`、`src-request`、`src-context`、`src-web` |
| `spring-framework` 图谱 | 115,037 节点、1,029,168 边、9,680 文件；主要 Web 社区为 `web-context`、`support-request`、`support-message`、`core-data` |
| `vernal-web` 当前 | 15 个 Rust 源文件（含 `lib.rs`），以框架中立合同为主；已有请求上下文、作用域、路由元数据、处理器调用、问题详情、安全主体、传输类型等对象 |
| Spring Web 当前规模 | `http` 31 个、`http.client` 35 个、`http.codec` 29 个、`http.converter` 23 个；`web` 全域 448 个 Java 文件。迁移目标应先覆盖公共基础层，再按适配器扩展。 |
| 已识别关键流程 | Spring：`getBody`、`writeTo`、`buildRequest`、`toEntity`、`BaseDefaultCodecs`、消息读写器/编码器；Vernal：`on_request`、`from_request`、`on_response`、`intercept`、`handle`。 |

## 二、总目标与边界

### 目标

1. 提供统一的 HTTP 方法、状态码、媒体类型、请求/响应、头、Cookie、实体、范围与问题详情模型。
2. 提供请求生命周期：路由匹配 → 请求上下文 → 提取/绑定 → Handler 调用 → 响应编码 → 错误映射。
3. 提供可插拔消息转换器（JSON、文本、表单、Multipart、字节流、SSE）及异步流式读写。
4. 提供客户端抽象（请求工厂、执行链、拦截器、响应读取）并以 Hyper/Reqwest 等 Rust 生态实现适配器。
5. 保持与 `vernal-context`、`vernal-beans`、`vernal-aop` 和 `vernal-expression` 的上下文、依赖注入、拦截和表达式绑定语义一致。

### 不直接迁移的 Java 专有机制

| Spring 机制 | Rust 化替代 |
|---|---|
| Servlet API、`WebApplicationInitializer` | Hyper/Tower/Axum/Actix/Salvo 等 Transport Adapter |
| Java 反射参数解析 | 显式 `FromRequest`/`IntoResponse`/`Extractor` trait 与宏 |
| Reactor `Publisher`/`Flux` | `futures::Stream`、`Body`、`BoxStream` |
| 字节码编译、反射优化 | Rust 编译期泛型、宏和静态分发 |
| Java Bean Validation 运行时扫描 | 类型化 extractor + 可选 `validator` 集成 |

## 三、阶段总览

| 阶段 | 内容 | 状态 | 主要产物 | 验收 |
|---|---|---|---|---|
| W0 | 基线、对象表、语义表、命名检查 | ✅ | 本目录四份文档 | 清单可追踪 |
| W1 | HTTP 核心值对象与错误 | ⬜ | method/status/media/header/entity/cookie/problem | 单元测试 + serde/解析测试 |
| W2 | 请求/响应抽象与生命周期 | ⬜ | request/response/input/output/exchange/context | 端到端请求流测试 |
| W3 | 路由、Handler、Extractor、Result | ⬜ | route/handler/handler_adapter/argument_resolver | 路由与参数绑定测试 |
| W4 | 消息转换器与 Codec | ⬜ | reader/writer/encoder/decoder/json/form/multipart | Content-Type/Accept 矩阵测试 |
| W5 | Middleware、过滤器、CORS、日志、请求 ID | ⬜ | filter/interceptor/cors/observation | 顺序、短路、错误传播测试 |
| W6 | 客户端抽象 | ⬜ | client request/response/factory/execution/interceptor | Mock transport + 重试/超时测试 |
| W7 | 多运行时适配器 | ⬜ | hyper/axum/actix/salvo/ntex/tide/warp/poem | 每适配器合同测试 |
| W8 | WebSocket、SSE、Range、静态资源 | ⬜ | websocket/sse/range/resource | 流式与断开测试 |
| W9 | Vernal 集成与表达式绑定 | ⬜ | `vernal-context` bridge、表达式 extractor | Bean/环境/表达式集成测试 |
| W10 | 文档、兼容矩阵、命名收尾 | ⬜ | API 文档、迁移指南、覆盖率报告 | 中文注释与清单 100% |

## 四、详细路线

### W1：HTTP 核心（P0）

目录建议：`src/http/`。

- `HttpMethod`、`HttpStatusCode`、`HttpStatus`、`MediaType`、`HttpHeaders`、`HttpCookie`。
- `HttpMessage`、`HttpInputMessage`、`HttpOutputMessage`、`HttpEntity<T>`、`RequestEntity<T>`、`ResponseEntity<T>`。
- `ContentDisposition`、`HttpRange`、`ETag`、`CacheControl`、`ProblemDetail`。
- 统一 `WebError`：解析错误、媒体类型不支持、请求体过大、绑定失败、超时、传输错误、处理器错误。

验收：大小写不敏感 Header、多值 Header、媒体类型参数/通配符、状态码 reason、实体构建器、RFC 兼容解析。

### W2-W3：服务端请求管线（P0）

将当前 `RequestContext`、`WebRequestScope`、`RouteMetadata`、`HandlerInvocation` 提升为核心管线对象：

```text
TransportRequest
  -> ServerWebExchange
  -> RouteMatcher / HandlerMapping
  -> HandlerAdapter
  -> ArgumentResolver / FromRequest
  -> Handler
  -> ReturnValueHandler / IntoResponse
  -> MessageWriter
  -> TransportResponse
```

要求：异步、可取消、请求体只能消费一次但可显式缓存、响应头在提交前可修改、错误统一映射为 `ProblemDetail`。

### W4：消息读写与编码（P0）

优先级：Bytes/Text → JSON → Form → Multipart → SSE → Protobuf/XML。

每个 Reader/Writer 必须声明：支持的 Rust 类型、媒体类型、最大内存、是否流式、错误类型、是否可重复读取。禁止将所有编码器塞入 `lib.rs`；每个公开对象独立文件。

### W5：横切 Web 能力（P1）

实现 `WebFilter`/`HandlerInterceptor` 语义：顺序、前置/后置、短路响应、异常传播、请求属性、日志脱敏、CORS、Request ID、观测上下文。与现有 `vernal-aop` 的 `intercept` 仅通过明确桥接集成，不让 Web 核心依赖具体运行时。

### W6-W8：客户端与协议扩展（P1/P2）

先实现 transport-neutral Client API，再分别接入 Hyper/Reqwest；之后实现 SSE、WebSocket、Range、静态资源、Multipart 上传下载。每项复用 W1 的实体、Header、MediaType 和 W4 的 Codec。

### W9-W10：集成与收尾

- `vernal-expression`：支持从 `RequestContext`、路由变量、Header、Query、Body 注册属性访问器。
- `vernal-context`：支持环境、Bean、作用域和请求级依赖注入。
- 所有公开 Rust 类型/方法补中文文档；生成 Spring→Vernal 缺口报告。

## 五、验收门槛

- 每个 P0 对象至少 2 个单元测试；每个 Codec 至少覆盖成功、错误、空体、超限、Content-Type 不匹配。
- 每个运行时适配器通过同一份 Web Contract Test。
- 关键流必须覆盖取消、超时、断开、重复读取、短路和异常映射。
- `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`。
- 文档对象表、语义表、命名表的状态必须与源码同步；禁止以“语义存在但对象缺失”标记完成。
