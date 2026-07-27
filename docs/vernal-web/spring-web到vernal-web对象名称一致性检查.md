# vernal-web 对象名称一致性检查

> 检查时间：2026-07-27
> 基线：Spring Framework 7.0.8 `spring-web`；CodeGraph 已索引 `spring-framework` 与 `vernal-framework`。
> 目的：检查 Spring Web 对象是否有稳定、无歧义、符合 Vernal 规范的 Rust 名称；本表不是声称对象已经实现。

## 一、检查结论

| 维度 | 结果 |
|---|---|
| 当前 Vernal Web Rust 文件 | 15（含 `lib.rs`） |
| Spring `http` 一级对象 | 31 |
| Spring `http.client` 一级对象 | 35 |
| Spring `http.codec` 一级对象 | 29 |
| Spring `http.converter` 一级对象 | 23 |
| Spring `web` 全域对象 | 448（包含 Servlet/WebFlux/WebSocket/client 等实现） |
| 当前直接名称对齐 | 已有 Vernal 合同对象：`RequestContext`、`RouteMetadata`、`HandlerInvocation`、`ProblemDetails`、`RequestId`、`SecurityPrincipal`、`TransportKind` 等 |
| 主要问题 | HTTP 核心值对象、消息 reader/writer、codec、客户端和运行时适配器尚未形成独立对象；不能把当前 15 文件视为 Spring Web 已迁移完成 |

## 二、当前对象名称检查

| Vernal 文件 | 类型 | 命名检查 | 结论/动作 |
|---|---|---|---|
| `request_context.rs` | `RequestContext` | ✅ PascalCase ↔ snake_case | 保留；扩展 HTTP 只读视图或组合 `ServerWebExchange` |
| `web_request_scope.rs` | `WebRequestScope` | ✅ | 保留 |
| `web_request_scope_owner.rs` | `WebRequestScopeOwner` | ✅ | 保留 |
| `route_metadata.rs` | `RouteMetadata` | ✅ | 保留；补 `HandlerMapping` 语义 |
| `handler_invocation.rs` | `HandlerInvocation` | ✅ | 保留；补 `HandlerAdapter` 关联 |
| `problem_details.rs` | `ProblemDetails` | 🔶 复数与 Spring `ProblemDetail` 不一致 | 建议新增规范对象 `ProblemDetail`，保留 `ProblemDetails` 作为兼容别名/集合，禁止二者语义混淆 |
| `problem_kind.rs` | `ProblemKind` | 🆕 Vernal 特有 | 保留，作为错误分类 enum |
| `web_failure.rs` | `WebFailure` | 🆕 Vernal 特有 | 保留，统一 transport/handler/codec 错误 |
| `request_id.rs` | `RequestId` | 🆕/✅ | 保留 |
| `security_principal.rs` | `SecurityPrincipal` | 🔶 Spring 更常用 `Principal` | 保留 Vernal 前缀，避免与通用 trait 冲突；可提供 `Principal` trait bridge |
| `context_carrier.rs` | `ContextCarrier` | 🆕 | 保留；命名表达请求上下文传播 |
| `transport_kind.rs` | `TransportKind` | 🆕 | 保留 |
| `integration_descriptor.rs` | `IntegrationDescriptor` | 🆕 | 保留 |
| `integration_role.rs` | `IntegrationRole` | 🆕 | 保留 |

## 三、必须采用的 Spring 对齐名称

### HTTP 核心

`HttpMethod` → `http_method.rs`；`HttpStatusCode` → `http_status_code.rs`；`HttpStatus` → `http_status.rs`；`MediaType` → `media_type.rs`；`HttpHeaders` → `http_headers.rs`；`HttpCookie` → `http_cookie.rs`；`ResponseCookie` → `response_cookie.rs`；`HttpMessage` → `http_message.rs`；`HttpInputMessage` → `http_input_message.rs`；`HttpOutputMessage` → `http_output_message.rs`；`HttpEntity` → `http_entity.rs`；`RequestEntity` → `request_entity.rs`；`ResponseEntity` → `response_entity.rs`；`ProblemDetail` → `problem_detail.rs`；`ContentDisposition` → `content_disposition.rs`；`HttpRange` → `http_range.rs`；`ETag` → `etag.rs`；`CacheControl` → `cache_control.rs`。

### 服务端管线

`ServerWebExchange` → `server_web_exchange.rs`；`ServerRequest` → `server_request.rs`；`ServerResponse` → `server_response.rs`；`WebHandler` → `web_handler.rs`；`WebFilter` → `web_filter.rs`；`HandlerMapping` → `handler_mapping.rs`；`HandlerAdapter` → `handler_adapter.rs`；`HandlerResult` → `handler_result.rs`；`HandlerMethod` → `handler_method.rs`；`ArgumentResolver` → `argument_resolver.rs`；`ReturnValueHandler` → `return_value_handler.rs`；`WebExceptionHandler` → `web_exception_handler.rs`。

### Codec

`HttpMessageReader` → `http_message_reader.rs`；`HttpMessageWriter` → `http_message_writer.rs`；`Decoder` → `decoder.rs`；`Encoder` → `encoder.rs`；`DataBuffer` → `data_buffer.rs`；`DataBufferFactory` → `data_buffer_factory.rs`；`CodecConfigurer` → `codec_configurer.rs`；`JsonCodec` → `json_codec.rs`；`FormCodec` → `form_codec.rs`；`MultipartCodec` → `multipart_codec.rs`；`SseCodec` → `sse_codec.rs`；`ServerSentEvent` → `server_sent_event.rs`。

### 客户端

`ClientHttpRequest` → `client_http_request.rs`；`ClientHttpResponse` → `client_http_response.rs`；`ClientHttpRequestFactory` → `client_http_request_factory.rs`；`ClientHttpRequestExecution` → `client_http_request_execution.rs`；`ClientHttpRequestInterceptor` → `client_http_request_interceptor.rs`；`HttpAccessor` → `http_accessor.rs`；`WebClient` → `client.rs`；请求/响应规格 → `client_spec.rs`。

## 四、名称冲突与合并政策

| 情况 | 政策 |
|---|---|
| Spring 同一对象有 Servlet 与 WebFlux 实现 | 核心只保留一个语义对象；具体实现使用 `adapters/servlet`、`adapters/hyper` 等 |
| `HttpRequest`、`ClientHttpRequest`、`ServerRequest` | 不合并；分别表达通用 HTTP request、客户端 transport request、服务端高层 request |
| `HttpResponse`、`ClientHttpResponse`、`ServerResponse` | 不合并；分别表达 transport、客户端、服务端语义 |
| `ProblemDetail` 与已有 `ProblemDetails` | 新增单数规范对象；旧复数类型仅作兼容集合/别名，必须标注 deprecated 计划 |
| 多个 JSON 实现（Jackson/Gson/Kotlin） | 核心统一 `JsonCodec` trait；实现以 feature/adapters 命名，不复制 Java 库名到核心类型 |
| `DataBuffer` 与 `Bytes` | `DataBuffer` 表示可计数、可释放/切片的 Web buffer；`Bytes` 是基础 payload，不得混为同一公开语义 |
| `WebFilter` 与 `HandlerInterceptor` | 分别保留；可通过 bridge 转换，但不强制同名 |
| `RequestContext` 与 `ServerWebExchange` | `RequestContext` 保留为 Vernal 请求作用域合同；`ServerWebExchange` 作为 Spring 对齐的 HTTP exchange façade |

## 五、命名验收规则

1. 文件名一律 snake_case，类型名 PascalCase，方法名 snake_case。
2. 一个公开对象一个文件；`lib.rs`/`mod.rs` 只做声明和导出。
3. Spring 名称是公共语义契约时保持一致；运行时、库实现、Java 生态专有名称进入 adapter，不污染核心。
4. 缩写统一：`Http`、`Uri`、`Sse`、`Json`、`Xml`、`Id`；不在同一 crate 混用 `HTTP`/`Http`、`URI`/`Uri`、`SSE`/`Sse`。
5. 类型名称不得只用过宽的 `Request`、`Response`、`Error`；必须携带层次前缀或明确泛型语义。
6. 每次新增对象必须同时更新：对象级对照表、语义迁移表、本文状态和测试清单。
7. “功能已经存在”不能替代“对象名称已对齐”；合并对象必须在表中写明合并理由、公开别名和拆分计划。

## 六、当前优先修正项

1. 将 `ProblemDetails` 与 Spring `ProblemDetail` 的单复数语义分离。
2. 建立 `http/`、`server/`、`codec/`、`client/`、`adapters/` 目录，避免把新增对象堆入当前根目录。
3. 先补 HTTP 核心对象，再补 `HandlerAdapter`/`ArgumentResolver`/`ReturnValueHandler`，最后实现 Codec 和客户端。
4. 为每个 adapter 使用统一 Web Contract Test，防止不同 Rust Web 框架产生名称和行为漂移。
